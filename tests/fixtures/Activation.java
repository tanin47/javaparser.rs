package sun.rmi.server;

import com.sun.rmi.rmid.ExecOptionPermission;
import com.sun.rmi.rmid.ExecPermission;
import java.io.File;
import java.io.FileOutputStream;
import java.io.IOException;
import java.io.ObjectInput;
import java.io.ObjectInputStream;
import java.io.PrintStream;
import java.io.Serializable;
import java.lang.reflect.InvocationTargetException;
import java.lang.reflect.Method;
import java.net.InetAddress;
import java.net.ServerSocket;
import java.net.Socket;
import java.net.SocketAddress;
import java.net.SocketException;
import java.net.URL;
import java.nio.channels.Channel;
import java.nio.channels.ServerSocketChannel;
import java.nio.file.Files;
import java.rmi.AccessException;
import java.rmi.AlreadyBoundException;
import java.rmi.ConnectException;
import java.rmi.ConnectIOException;
import java.rmi.MarshalledObject;
import java.rmi.NoSuchObjectException;
import java.rmi.NotBoundException;
import java.rmi.Remote;
import java.rmi.RemoteException;
import java.rmi.activation.ActivationDesc;
import java.rmi.activation.ActivationException;
import java.rmi.activation.ActivationGroup;
import java.rmi.activation.ActivationGroupDesc;
import java.rmi.activation.ActivationGroupID;
import java.rmi.activation.ActivationID;
import java.rmi.activation.ActivationInstantiator;
import java.rmi.activation.ActivationMonitor;
import java.rmi.activation.ActivationSystem;
import java.rmi.activation.Activator;
import java.rmi.activation.UnknownGroupException;
import java.rmi.activation.UnknownObjectException;
import java.rmi.registry.Registry;
import java.rmi.server.ObjID;
import java.rmi.server.RMIClassLoader;
import java.rmi.server.RMIClientSocketFactory;
import java.rmi.server.RMIServerSocketFactory;
import java.rmi.server.RemoteObject;
import java.rmi.server.RemoteServer;
import java.rmi.server.UnicastRemoteObject;
import java.security.AccessControlException;
import java.security.AccessController;
import java.security.AllPermission;
import java.security.CodeSource;
import java.security.Permission;
import java.security.PermissionCollection;
import java.security.Permissions;
import java.security.Policy;
import java.security.PrivilegedAction;
import java.security.PrivilegedExceptionAction;
import java.security.cert.Certificate;
import java.text.MessageFormat;
import java.util.ArrayList;
import java.util.Arrays;
import java.util.Date;
import java.util.Enumeration;
import java.util.HashMap;
import java.util.HashSet;
import java.util.Iterator;
import java.util.Map;
import java.util.MissingResourceException;
import java.util.Properties;
import java.util.ResourceBundle;
import java.util.Set;
import java.util.concurrent.ConcurrentHashMap;
import sun.rmi.log.LogHandler;
import sun.rmi.log.ReliableLog;
import sun.rmi.registry.RegistryImpl;
import sun.rmi.transport.LiveRef;
import sun.security.action.GetBooleanAction;
import sun.security.action.GetIntegerAction;
import sun.security.action.GetPropertyAction;
import sun.security.provider.PolicyFile;

public class Activation implements Serializable {
  private static final long serialVersionUID = 2921265612698155191L;
  private static final byte MAJOR_VERSION = 1;
  private static final byte MINOR_VERSION = 0;
  private static Object execPolicy;
  private static Method execPolicyMethod;
  private static boolean debugExec;
  private Map<ActivationID, ActivationGroupID> idTable;
  private Map<ActivationGroupID, Activation.GroupEntry> groupTable;
  private byte majorVersion;
  private byte minorVersion;
  private transient int groupSemaphore;
  private transient int groupCounter;
  private transient ReliableLog log;
  private transient int numUpdates;
  private transient String[] command;
  private static final long groupTimeout = (long)getInt("sun.rmi.activation.groupTimeout", 60000);
  private static final int snapshotInterval = getInt("sun.rmi.activation.snapshotInterval", 200);
  private static final long execTimeout = (long)getInt("sun.rmi.activation.execTimeout", 30000);
  private static final Object initLock = new Object();
  private static boolean initDone = false;
  private transient Activator activator;
  private transient Activator activatorStub;
  private transient ActivationSystem system;
  private transient ActivationSystem systemStub;
  private transient ActivationMonitor monitor;
  private transient Registry registry;
  private transient volatile boolean shuttingDown;
  private transient volatile Object startupLock;
  private transient Thread shutdownHook;
  private static ResourceBundle resources = null;

  private static int getInt(String var0, int var1) {
    return (Integer)AccessController.doPrivileged((PrivilegedAction)(new GetIntegerAction(var0, var1)));
  }

  private Activation() {
    this.idTable = new ConcurrentHashMap();
    this.groupTable = new ConcurrentHashMap();
    this.majorVersion = 1;
    this.minorVersion = 0;
    this.shuttingDown = false;
  }

  private static void startActivation(int var0, RMIServerSocketFactory var1, String var2, String[] var3) throws Exception {
    ReliableLog var4 = new ReliableLog(var2, new Activation.ActLogHandler());
    Activation var5 = (Activation)var4.recover();
    var5.init(var0, var1, var4, var3);
  }

  private void init(int var1, RMIServerSocketFactory var2, ReliableLog var3, String[] var4) throws Exception {
    this.log = var3;
    this.numUpdates = 0;
    this.shutdownHook = new Activation.ShutdownHook();
    this.groupSemaphore = getInt("sun.rmi.activation.groupThrottle", 3);
    this.groupCounter = 0;
    Runtime.getRuntime().addShutdownHook(this.shutdownHook);
    ActivationGroupID[] var5 = (ActivationGroupID[])this.groupTable.keySet().toArray(new ActivationGroupID[0]);
    synchronized(this.startupLock = new Object()) {
      this.activator = new Activation.ActivatorImpl(var1, var2);
      this.activatorStub = (Activator)RemoteObject.toStub(this.activator);
      this.system = new Activation.ActivationSystemImpl(var1, var2);
      this.systemStub = (ActivationSystem)RemoteObject.toStub(this.system);
      this.monitor = new Activation.ActivationMonitorImpl(var1, var2);
      this.initCommand(var4);
      this.registry = new Activation.SystemRegistryImpl(var1, (RMIClientSocketFactory)null, var2, this.systemStub);
      if (var2 != null) {
        synchronized(initLock) {
          initDone = true;
          initLock.notifyAll();
        }
      }
    }

    this.startupLock = null;
    int var6 = var5.length;

    while(true) {
      --var6;
      if (var6 < 0) {
        return;
      }

      try {
        this.getGroupEntry(var5[var6]).restartServices();
      } catch (UnknownGroupException var10) {
        System.err.println(getTextResource("rmid.restart.group.warning"));
        var10.printStackTrace();
      }
    }
  }

  private void readObject(ObjectInputStream var1) throws IOException, ClassNotFoundException {
    var1.defaultReadObject();
    if (!(this.groupTable instanceof ConcurrentHashMap)) {
      this.groupTable = new ConcurrentHashMap(this.groupTable);
    }

    if (!(this.idTable instanceof ConcurrentHashMap)) {
      this.idTable = new ConcurrentHashMap(this.idTable);
    }

  }

  private void checkShutdown() throws ActivationException {
    Object var1 = this.startupLock;
    if (var1 != null) {
      synchronized(var1) {
        ;
      }
    }

    if (this.shuttingDown) {
      throw new ActivationException("activation system shutting down");
    }
  }

  private static void unexport(Remote var0) {
    while(true) {
      try {
        if (UnicastRemoteObject.unexportObject(var0, false)) {
          return;
        }

        Thread.sleep(100L);
      } catch (Exception var2) {
      }
    }
  }

  private ActivationGroupID getGroupID(ActivationID var1) throws UnknownObjectException {
    ActivationGroupID var2 = (ActivationGroupID)this.idTable.get(var1);
    if (var2 != null) {
      return var2;
    } else {
      throw new UnknownObjectException("unknown object: " + var1);
    }
  }

  private Activation.GroupEntry getGroupEntry(ActivationGroupID var1, boolean var2) throws UnknownGroupException {
    if (var1.getClass() == ActivationGroupID.class) {
      Activation.GroupEntry var3;
      if (var2) {
        var3 = (Activation.GroupEntry)this.groupTable.remove(var1);
      } else {
        var3 = (Activation.GroupEntry)this.groupTable.get(var1);
      }

      if (var3 != null && !var3.removed) {
        return var3;
      }
    }

    throw new UnknownGroupException("group unknown");
  }

  private Activation.GroupEntry getGroupEntry(ActivationGroupID var1) throws UnknownGroupException {
    return this.getGroupEntry(var1, false);
  }

  private Activation.GroupEntry removeGroupEntry(ActivationGroupID var1) throws UnknownGroupException {
    return this.getGroupEntry(var1, true);
  }

  private Activation.GroupEntry getGroupEntry(ActivationID var1) throws UnknownObjectException {
    ActivationGroupID var2 = this.getGroupID(var1);
    Activation.GroupEntry var3 = (Activation.GroupEntry)this.groupTable.get(var2);
    if (var3 != null && !var3.removed) {
      return var3;
    } else {
      throw new UnknownObjectException("object's group removed");
    }
  }

  private String[] activationArgs(ActivationGroupDesc var1) {
    ActivationGroupDesc.CommandEnvironment var2 = var1.getCommandEnvironment();
    ArrayList var3 = new ArrayList();
    var3.add(var2 != null && var2.getCommandPath() != null ? var2.getCommandPath() : this.command[0]);
    if (var2 != null && var2.getCommandOptions() != null) {
      var3.addAll(Arrays.asList(var2.getCommandOptions()));
    }

    Properties var4 = var1.getPropertyOverrides();
    if (var4 != null) {
      Enumeration var5 = var4.propertyNames();

      while(var5.hasMoreElements()) {
        String var6 = (String)var5.nextElement();
        var3.add("-D" + var6 + "=" + var4.getProperty(var6));
      }
    }

    for(int var7 = 1; var7 < this.command.length; ++var7) {
      var3.add(this.command[var7]);
    }

    String[] var8 = new String[var3.size()];
    System.arraycopy(var3.toArray(), 0, var8, 0, var8.length);
    return var8;
  }

  private void checkArgs(ActivationGroupDesc var1, String[] var2) throws SecurityException, ActivationException {
    if (execPolicyMethod != null) {
      if (var2 == null) {
        var2 = this.activationArgs(var1);
      }

      try {
        execPolicyMethod.invoke(execPolicy, var1, var2);
      } catch (InvocationTargetException var5) {
        Throwable var4 = var5.getTargetException();
        if (var4 instanceof SecurityException) {
          throw (SecurityException)var4;
        }

        throw new ActivationException(execPolicyMethod.getName() + ": unexpected exception", var5);
      } catch (Exception var6) {
        throw new ActivationException(execPolicyMethod.getName() + ": unexpected exception", var6);
      }
    }

  }

  private void addLogRecord(Activation.LogRecord var1) throws ActivationException {
    synchronized(this.log) {
      this.checkShutdown();

      try {
        this.log.update(var1, true);
      } catch (Exception var8) {
        this.numUpdates = snapshotInterval;
        System.err.println(getTextResource("rmid.log.update.warning"));
        var8.printStackTrace();
      }

      if (++this.numUpdates >= snapshotInterval) {
        try {
          this.log.snapshot(this);
          this.numUpdates = 0;
        } catch (Exception var7) {
          System.err.println(getTextResource("rmid.log.snapshot.warning"));
          var7.printStackTrace();

          try {
            this.system.shutdown();
          } catch (RemoteException var6) {
          }

          throw new ActivationException("log snapshot failed", var7);
        }

      }
    }
  }

  private void initCommand(String[] var1) {
    this.command = new String[var1.length + 2];
    AccessController.doPrivileged(new PrivilegedAction<Void>() {
      public Void run() {
        try {
          Activation.this.command[0] = System.getProperty("java.home") + File.separator + "bin" + File.separator + "java";
        } catch (Exception var2) {
          System.err.println(Activation.getTextResource("rmid.unfound.java.home.property"));
          Activation.this.command[0] = "java";
        }

        return null;
      }
    });
    System.arraycopy(var1, 0, this.command, 1, var1.length);
    this.command[this.command.length - 1] = "sun.rmi.server.ActivationGroupInit";
  }

  private static void bomb(String var0) {
    System.err.println("rmid: " + var0);
    System.err.println(MessageFormat.format(getTextResource("rmid.usage"), "rmid"));
    System.exit(1);
  }

  public static void main(String[] var0) {
    boolean var1 = false;
    if (System.getSecurityManager() == null) {
      System.setSecurityManager(new SecurityManager());
    }

    try {
      final int var2 = 1098;
      Activation.ActivationServerSocketFactory var3 = null;
      Channel var4 = (Channel)AccessController.doPrivileged(new PrivilegedExceptionAction<Channel>() {
        public Channel run() throws IOException {
          return System.inheritedChannel();
        }
      });
      if (var4 != null && var4 instanceof ServerSocketChannel) {
        AccessController.doPrivileged(new PrivilegedExceptionAction<Void>() {
          public Void run() throws IOException {
            File var1 = Files.createTempFile("rmid-err", (String)null).toFile();
            PrintStream var2 = new PrintStream(new FileOutputStream(var1));
            System.setErr(var2);
            return null;
          }
        });
        ServerSocket var5 = ((ServerSocketChannel)var4).socket();
        var2 = var5.getLocalPort();
        var3 = new Activation.ActivationServerSocketFactory(var5);
        System.err.println((Object)(new Date()));
        System.err.println(getTextResource("rmid.inherited.channel.info") + ": " + var4);
      }

      String var14 = null;
      ArrayList var6 = new ArrayList();

      for(int var7 = 0; var7 < var0.length; ++var7) {
        if (var0[var7].equals("-port")) {
          if (var3 != null) {
            bomb(getTextResource("rmid.syntax.port.badarg"));
          }

          if (var7 + 1 < var0.length) {
            try {
              ++var7;
              var2 = Integer.parseInt(var0[var7]);
            } catch (NumberFormatException var10) {
              bomb(getTextResource("rmid.syntax.port.badnumber"));
            }
          } else {
            bomb(getTextResource("rmid.syntax.port.missing"));
          }
        } else if (var0[var7].equals("-log")) {
          if (var7 + 1 < var0.length) {
            ++var7;
            var14 = var0[var7];
          } else {
            bomb(getTextResource("rmid.syntax.log.missing"));
          }
        } else if (var0[var7].equals("-stop")) {
          var1 = true;
        } else if (var0[var7].startsWith("-C")) {
          var6.add(var0[var7].substring(2));
        } else {
          bomb(MessageFormat.format(getTextResource("rmid.syntax.illegal.option"), var0[var7]));
        }
      }

      if (var14 == null) {
        if (var3 != null) {
          bomb(getTextResource("rmid.syntax.log.required"));
        } else {
          var14 = "log";
        }
      }

      debugExec = (Boolean)AccessController.doPrivileged((PrivilegedAction)(new GetBooleanAction("sun.rmi.server.activation.debugExec")));
      String var15 = (String)AccessController.doPrivileged((PrivilegedAction)(new GetPropertyAction("sun.rmi.activation.execPolicy", (String)null)));
      if (var15 == null) {
        if (!var1) {
          Activation.DefaultExecPolicy.checkConfiguration();
        }

        var15 = "default";
      }

      if (!var15.equals("none")) {
        if (var15.equals("") || var15.equals("default")) {
          var15 = Activation.DefaultExecPolicy.class.getName();
        }

        try {
          Class var8 = getRMIClass(var15);
          execPolicy = var8.newInstance();
          execPolicyMethod = var8.getMethod("checkExecCommand", ActivationGroupDesc.class, String[].class);
        } catch (Exception var12) {
          if (debugExec) {
            System.err.println(getTextResource("rmid.exec.policy.exception"));
            var12.printStackTrace();
          }

          bomb(getTextResource("rmid.exec.policy.invalid"));
        }
      }

      if (var1) {
        AccessController.doPrivileged(new PrivilegedAction<Void>() {
          public Void run() {
            System.setProperty("java.rmi.activation.port", Integer.toString(var2));
            return null;
          }
        });
        ActivationSystem var9 = ActivationGroup.getSystem();
        var9.shutdown();
        System.exit(0);
      }

      startActivation(var2, var3, var14, (String[])var6.toArray(new String[var6.size()]));

      while(true) {
        while(true) {
          try {
            Thread.sleep(Long.MAX_VALUE);
          } catch (InterruptedException var11) {
          }
        }
      }
    } catch (Exception var13) {
      System.err.println(MessageFormat.format(getTextResource("rmid.unexpected.exception"), var13));
      var13.printStackTrace();
      System.exit(1);
    }
  }

  private static String getTextResource(String var0) {
    if (resources == null) {
      try {
        resources = ResourceBundle.getBundle("sun.rmi.server.resources.rmid");
      } catch (MissingResourceException var4) {
      }

      if (resources == null) {
        return "[missing resource file: " + var0 + "]";
      }
    }

    String var1 = null;

    try {
      var1 = resources.getString(var0);
    } catch (MissingResourceException var3) {
    }

    return var1 == null ? "[missing resource: " + var0 + "]" : var1;
  }

  private static Class<?> getRMIClass(String var0) throws Exception {
    return RMIClassLoader.loadClass(var0);
  }

  private synchronized String Pstartgroup() throws ActivationException {
    while(true) {
      this.checkShutdown();
      if (this.groupSemaphore > 0) {
        --this.groupSemaphore;
        return "Group-" + this.groupCounter++;
      }

      try {
        this.wait();
      } catch (InterruptedException var2) {
      }
    }
  }

  private synchronized void Vstartgroup() {
    ++this.groupSemaphore;
    this.notifyAll();
  }

  // $FF: synthetic method
  Activation(Object var1) {
    this();
  }

  private static class DelayedAcceptServerSocket extends ServerSocket {
    private final ServerSocket serverSocket;

    DelayedAcceptServerSocket(ServerSocket var1) throws IOException {
      this.serverSocket = var1;
    }

    public void bind(SocketAddress var1) throws IOException {
      this.serverSocket.bind(var1);
    }

    public void bind(SocketAddress var1, int var2) throws IOException {
      this.serverSocket.bind(var1, var2);
    }

    public InetAddress getInetAddress() {
      return (InetAddress)AccessController.doPrivileged(new PrivilegedAction<InetAddress>() {
        public InetAddress run() {
          return DelayedAcceptServerSocket.this.serverSocket.getInetAddress();
        }
      });
    }

    public int getLocalPort() {
      return this.serverSocket.getLocalPort();
    }

    public SocketAddress getLocalSocketAddress() {
      return (SocketAddress)AccessController.doPrivileged(new PrivilegedAction<SocketAddress>() {
        public SocketAddress run() {
          return DelayedAcceptServerSocket.this.serverSocket.getLocalSocketAddress();
        }
      });
    }

    public Socket accept() throws IOException {
      synchronized(Activation.initLock) {
        try {
          while(!Activation.initDone) {
            Activation.initLock.wait();
          }
        } catch (InterruptedException var4) {
          throw new AssertionError(var4);
        }
      }

      return this.serverSocket.accept();
    }

    public void close() throws IOException {
      this.serverSocket.close();
    }

    public ServerSocketChannel getChannel() {
      return this.serverSocket.getChannel();
    }

    public boolean isBound() {
      return this.serverSocket.isBound();
    }

    public boolean isClosed() {
      return this.serverSocket.isClosed();
    }

    public void setSoTimeout(int var1) throws SocketException {
      this.serverSocket.setSoTimeout(var1);
    }

    public int getSoTimeout() throws IOException {
      return this.serverSocket.getSoTimeout();
    }

    public void setReuseAddress(boolean var1) throws SocketException {
      this.serverSocket.setReuseAddress(var1);
    }

    public boolean getReuseAddress() throws SocketException {
      return this.serverSocket.getReuseAddress();
    }

    public String toString() {
      return this.serverSocket.toString();
    }

    public void setReceiveBufferSize(int var1) throws SocketException {
      this.serverSocket.setReceiveBufferSize(var1);
    }

    public int getReceiveBufferSize() throws SocketException {
      return this.serverSocket.getReceiveBufferSize();
    }
  }

  private static class ActivationServerSocketFactory implements RMIServerSocketFactory {
    private final ServerSocket serverSocket;

    ActivationServerSocketFactory(ServerSocket var1) {
      this.serverSocket = var1;
    }

    public ServerSocket createServerSocket(int var1) throws IOException {
      return new Activation.DelayedAcceptServerSocket(this.serverSocket);
    }
  }

  public static class DefaultExecPolicy {
    public void checkExecCommand(ActivationGroupDesc var1, String[] var2) throws SecurityException {
      PermissionCollection var3 = getExecPermissions();
      Properties var4 = var1.getPropertyOverrides();
      String var7;
      if (var4 != null) {
        Enumeration var5 = var4.propertyNames();

        while(var5.hasMoreElements()) {
          String var6 = (String)var5.nextElement();
          var7 = var4.getProperty(var6);
          String var8 = "-D" + var6 + "=" + var7;

          try {
            checkPermission(var3, new ExecOptionPermission(var8));
          } catch (AccessControlException var13) {
            if (!var7.equals("")) {
              throw var13;
            }

            checkPermission(var3, new ExecOptionPermission("-D" + var6));
          }
        }
      }

      String var14 = var1.getClassName();
      if ((var14 == null || var14.equals(ActivationGroupImpl.class.getName())) && var1.getLocation() == null && var1.getData() == null) {
        ActivationGroupDesc.CommandEnvironment var15 = var1.getCommandEnvironment();
        if (var15 != null) {
          var7 = var15.getCommandPath();
          if (var7 != null) {
            checkPermission(var3, new ExecPermission(var7));
          }

          String[] var16 = var15.getCommandOptions();
          if (var16 != null) {
            String[] var9 = var16;
            int var10 = var16.length;

            for(int var11 = 0; var11 < var10; ++var11) {
              String var12 = var9[var11];
              checkPermission(var3, new ExecOptionPermission(var12));
            }
          }
        }

      } else {
        throw new AccessControlException("access denied (custom group implementation not allowed)");
      }
    }

    static void checkConfiguration() {
      Policy var0 = (Policy)AccessController.doPrivileged(new PrivilegedAction<Policy>() {
        public Policy run() {
          return Policy.getPolicy();
        }
      });
      if (var0 instanceof PolicyFile) {
        PermissionCollection var1 = getExecPermissions();
        Enumeration var2 = var1.elements();

        Permission var3;
        do {
          if (!var2.hasMoreElements()) {
            System.err.println(Activation.getTextResource("rmid.exec.perms.inadequate"));
            return;
          }

          var3 = (Permission)var2.nextElement();
        } while(!(var3 instanceof AllPermission) && !(var3 instanceof ExecPermission) && !(var3 instanceof ExecOptionPermission));

      }
    }

    private static PermissionCollection getExecPermissions() {
      PermissionCollection var0 = (PermissionCollection)AccessController.doPrivileged(new PrivilegedAction<PermissionCollection>() {
        public PermissionCollection run() {
          CodeSource var1 = new CodeSource((URL)null, (Certificate[])null);
          Policy var2 = Policy.getPolicy();
          return (PermissionCollection)(var2 != null ? var2.getPermissions(var1) : new Permissions());
        }
      });
      return var0;
    }

    private static void checkPermission(PermissionCollection var0, Permission var1) throws AccessControlException {
      if (!var0.implies(var1)) {
        throw new AccessControlException("access denied " + var1.toString());
      }
    }
  }

  private static class LogGroupIncarnation extends Activation.LogRecord {
    private static final long serialVersionUID = 4146872747377631897L;
    private ActivationGroupID id;
    private long inc;

    LogGroupIncarnation(ActivationGroupID var1, long var2) {
      super(null);
      this.id = var1;
      this.inc = var2;
    }

    Object apply(Object var1) {
      try {
        Activation.GroupEntry var2 = ((Activation)var1).getGroupEntry(this.id);
        var2.incarnation = this.inc;
      } catch (Exception var3) {
        System.err.println(MessageFormat.format(Activation.getTextResource("rmid.log.recover.warning"), "LogGroupIncarnation"));
        var3.printStackTrace();
      }

      return var1;
    }
  }

  private static class LogUnregisterGroup extends Activation.LogRecord {
    private static final long serialVersionUID = -3356306586522147344L;
    private ActivationGroupID id;

    LogUnregisterGroup(ActivationGroupID var1) {
      super(null);
      this.id = var1;
    }

    Object apply(Object var1) {
      Activation.GroupEntry var2 = (Activation.GroupEntry)((Activation)var1).groupTable.remove(this.id);

      try {
        var2.unregisterGroup(false);
      } catch (Exception var4) {
        System.err.println(MessageFormat.format(Activation.getTextResource("rmid.log.recover.warning"), "LogUnregisterGroup"));
        var4.printStackTrace();
      }

      return var1;
    }
  }

  private static class LogUpdateGroupDesc extends Activation.LogRecord {
    private static final long serialVersionUID = -1271300989218424337L;
    private ActivationGroupID id;
    private ActivationGroupDesc desc;

    LogUpdateGroupDesc(ActivationGroupID var1, ActivationGroupDesc var2) {
      super(null);
      this.id = var1;
      this.desc = var2;
    }

    Object apply(Object var1) {
      try {
        ((Activation)var1).getGroupEntry(this.id).setActivationGroupDesc(this.id, this.desc, false);
      } catch (Exception var3) {
        System.err.println(MessageFormat.format(Activation.getTextResource("rmid.log.recover.warning"), "LogUpdateGroupDesc"));
        var3.printStackTrace();
      }

      return var1;
    }
  }

  private static class LogUpdateDesc extends Activation.LogRecord {
    private static final long serialVersionUID = 545511539051179885L;
    private ActivationID id;
    private ActivationDesc desc;

    LogUpdateDesc(ActivationID var1, ActivationDesc var2) {
      super(null);
      this.id = var1;
      this.desc = var2;
    }

    Object apply(Object var1) {
      try {
        ((Activation)var1).getGroupEntry(this.id).setActivationDesc(this.id, this.desc, false);
      } catch (Exception var3) {
        System.err.println(MessageFormat.format(Activation.getTextResource("rmid.log.recover.warning"), "LogUpdateDesc"));
        var3.printStackTrace();
      }

      return var1;
    }
  }

  private static class LogRegisterGroup extends Activation.LogRecord {
    private static final long serialVersionUID = -1966827458515403625L;
    private ActivationGroupID id;
    private ActivationGroupDesc desc;

    LogRegisterGroup(ActivationGroupID var1, ActivationGroupDesc var2) {
      super(null);
      this.id = var1;
      this.desc = var2;
    }

    Object apply(Object var1) {
      Map var10000 = ((Activation)var1).groupTable;
      ActivationGroupID var10001 = this.id;
      Activation var10004 = (Activation)var1;
      ((Activation)var1).getClass();
      var10000.put(var10001, var10004.new GroupEntry(this.id, this.desc));
      return var1;
    }
  }

  private static class LogUnregisterObject extends Activation.LogRecord {
    private static final long serialVersionUID = 6269824097396935501L;
    private ActivationID id;

    LogUnregisterObject(ActivationID var1) {
      super(null);
      this.id = var1;
    }

    Object apply(Object var1) {
      try {
        ((Activation)var1).getGroupEntry(this.id).unregisterObject(this.id, false);
      } catch (Exception var3) {
        System.err.println(MessageFormat.format(Activation.getTextResource("rmid.log.recover.warning"), "LogUnregisterObject"));
        var3.printStackTrace();
      }

      return var1;
    }
  }

  private static class LogRegisterObject extends Activation.LogRecord {
    private static final long serialVersionUID = -6280336276146085143L;
    private ActivationID id;
    private ActivationDesc desc;

    LogRegisterObject(ActivationID var1, ActivationDesc var2) {
      super(null);
      this.id = var1;
      this.desc = var2;
    }

    Object apply(Object var1) {
      try {
        ((Activation)var1).getGroupEntry(this.desc.getGroupID()).registerObject(this.id, this.desc, false);
      } catch (Exception var3) {
        System.err.println(MessageFormat.format(Activation.getTextResource("rmid.log.recover.warning"), "LogRegisterObject"));
        var3.printStackTrace();
      }

      return var1;
    }
  }

  private abstract static class LogRecord implements Serializable {
    private static final long serialVersionUID = 8395140512322687529L;

    private LogRecord() {
    }

    abstract Object apply(Object var1) throws Exception;

    // $FF: synthetic method
    LogRecord(Object var1) {
      this();
    }
  }

  private static class ActLogHandler extends LogHandler {
    ActLogHandler() {
    }

    public Object initialSnapshot() {
      return new Activation();
    }

    public Object applyUpdate(Object var1, Object var2) throws Exception {
      return ((Activation.LogRecord)var1).apply(var2);
    }
  }

  private static class ObjectEntry implements Serializable {
    private static final long serialVersionUID = -5500114225321357856L;
    ActivationDesc desc;
    transient volatile MarshalledObject<? extends Remote> stub = null;
    transient volatile boolean removed = false;

    ObjectEntry(ActivationDesc var1) {
      this.desc = var1;
    }

    synchronized MarshalledObject<? extends Remote> activate(ActivationID var1, boolean var2, ActivationInstantiator var3) throws RemoteException, ActivationException {
      MarshalledObject var4 = this.stub;
      if (this.removed) {
        throw new UnknownObjectException("object removed");
      } else if (!var2 && var4 != null) {
        return var4;
      } else {
        var4 = var3.newInstance(var1, this.desc);
        this.stub = var4;
        return var4;
      }
    }

    void reset() {
      this.stub = null;
    }
  }

  private class GroupEntry implements Serializable {
    private static final long serialVersionUID = 7222464070032993304L;
    private static final int MAX_TRIES = 2;
    private static final int NORMAL = 0;
    private static final int CREATING = 1;
    private static final int TERMINATE = 2;
    private static final int TERMINATING = 3;
    ActivationGroupDesc desc = null;
    ActivationGroupID groupID = null;
    long incarnation = 0L;
    Map<ActivationID, Activation.ObjectEntry> objects = new HashMap();
    Set<ActivationID> restartSet = new HashSet();
    transient ActivationInstantiator group = null;
    transient int status = 0;
    transient long waitTime = 0L;
    transient String groupName = null;
    transient Process child = null;
    transient boolean removed = false;
    transient Activation.GroupEntry.Watchdog watchdog = null;

    GroupEntry(ActivationGroupID var2, ActivationGroupDesc var3) {
      this.groupID = var2;
      this.desc = var3;
    }

    void restartServices() {
      Iterator var1 = null;
      synchronized(this) {
        if (this.restartSet.isEmpty()) {
          return;
        }

        var1 = (new HashSet(this.restartSet)).iterator();
      }

      while(var1.hasNext()) {
        ActivationID var2 = (ActivationID)var1.next();

        try {
          this.activate(var2, true);
        } catch (Exception var5) {
          if (Activation.this.shuttingDown) {
            return;
          }

          System.err.println(Activation.getTextResource("rmid.restart.service.warning"));
          var5.printStackTrace();
        }
      }

    }

    synchronized void activeGroup(ActivationInstantiator var1, long var2) throws ActivationException, UnknownGroupException {
      if (this.incarnation != var2) {
        throw new ActivationException("invalid incarnation");
      } else if (this.group != null) {
        if (!this.group.equals(var1)) {
          throw new ActivationException("group already active");
        }
      } else if (this.child != null && this.status != 1) {
        throw new ActivationException("group not being created");
      } else {
        this.group = var1;
        this.status = 0;
        this.notifyAll();
      }
    }

    private void checkRemoved() throws UnknownGroupException {
      if (this.removed) {
        throw new UnknownGroupException("group removed");
      }
    }

    private Activation.ObjectEntry getObjectEntry(ActivationID var1) throws UnknownObjectException {
      if (this.removed) {
        throw new UnknownObjectException("object's group removed");
      } else {
        Activation.ObjectEntry var2 = (Activation.ObjectEntry)this.objects.get(var1);
        if (var2 == null) {
          throw new UnknownObjectException("object unknown");
        } else {
          return var2;
        }
      }
    }

    synchronized void registerObject(ActivationID var1, ActivationDesc var2, boolean var3) throws UnknownGroupException, ActivationException {
      this.checkRemoved();
      this.objects.put(var1, new Activation.ObjectEntry(var2));
      if (var2.getRestartMode()) {
        this.restartSet.add(var1);
      }

      Activation.this.idTable.put(var1, this.groupID);
      if (var3) {
        Activation.this.addLogRecord(new Activation.LogRegisterObject(var1, var2));
      }

    }

    synchronized void unregisterObject(ActivationID var1, boolean var2) throws UnknownGroupException, ActivationException {
      Activation.ObjectEntry var3 = this.getObjectEntry(var1);
      var3.removed = true;
      this.objects.remove(var1);
      if (var3.desc.getRestartMode()) {
        this.restartSet.remove(var1);
      }

      Activation.this.idTable.remove(var1);
      if (var2) {
        Activation.this.addLogRecord(new Activation.LogUnregisterObject(var1));
      }

    }

    synchronized void unregisterGroup(boolean var1) throws UnknownGroupException, ActivationException {
      this.checkRemoved();
      this.removed = true;

      Activation.ObjectEntry var5;
      for(Iterator var2 = this.objects.entrySet().iterator(); var2.hasNext(); var5.removed = true) {
        Map.Entry var3 = (Map.Entry)var2.next();
        ActivationID var4 = (ActivationID)var3.getKey();
        Activation.this.idTable.remove(var4);
        var5 = (Activation.ObjectEntry)var3.getValue();
      }

      this.objects.clear();
      this.restartSet.clear();
      this.reset();
      this.childGone();
      if (var1) {
        Activation.this.addLogRecord(new Activation.LogUnregisterGroup(this.groupID));
      }

    }

    synchronized ActivationDesc setActivationDesc(ActivationID var1, ActivationDesc var2, boolean var3) throws UnknownObjectException, UnknownGroupException, ActivationException {
      Activation.ObjectEntry var4 = this.getObjectEntry(var1);
      ActivationDesc var5 = var4.desc;
      var4.desc = var2;
      if (var2.getRestartMode()) {
        this.restartSet.add(var1);
      } else {
        this.restartSet.remove(var1);
      }

      if (var3) {
        Activation.this.addLogRecord(new Activation.LogUpdateDesc(var1, var2));
      }

      return var5;
    }

    synchronized ActivationDesc getActivationDesc(ActivationID var1) throws UnknownObjectException, UnknownGroupException {
      return this.getObjectEntry(var1).desc;
    }

    synchronized ActivationGroupDesc setActivationGroupDesc(ActivationGroupID var1, ActivationGroupDesc var2, boolean var3) throws UnknownGroupException, ActivationException {
      this.checkRemoved();
      ActivationGroupDesc var4 = this.desc;
      this.desc = var2;
      if (var3) {
        Activation.this.addLogRecord(new Activation.LogUpdateGroupDesc(var1, var2));
      }

      return var4;
    }

    synchronized void inactiveGroup(long var1, boolean var3) throws UnknownGroupException {
      this.checkRemoved();
      if (this.incarnation != var1) {
        throw new UnknownGroupException("invalid incarnation");
      } else {
        this.reset();
        if (var3) {
          this.terminate();
        } else if (this.child != null && this.status == 0) {
          this.status = 2;
          this.watchdog.noRestart();
        }

      }
    }

    synchronized void activeObject(ActivationID var1, MarshalledObject<? extends Remote> var2) throws UnknownObjectException {
      this.getObjectEntry(var1).stub = var2;
    }

    synchronized void inactiveObject(ActivationID var1) throws UnknownObjectException {
      this.getObjectEntry(var1).reset();
    }

    private synchronized void reset() {
      this.group = null;
      Iterator var1 = this.objects.values().iterator();

      while(var1.hasNext()) {
        Activation.ObjectEntry var2 = (Activation.ObjectEntry)var1.next();
        var2.reset();
      }

    }

    private void childGone() {
      if (this.child != null) {
        this.child = null;
        this.watchdog.dispose();
        this.watchdog = null;
        this.status = 0;
        this.notifyAll();
      }

    }

    private void terminate() {
      if (this.child != null && this.status != 3) {
        this.child.destroy();
        this.status = 3;
        this.waitTime = System.currentTimeMillis() + Activation.groupTimeout;
        this.notifyAll();
      }

    }

    private void await() {
      while(true) {
        switch(this.status) {
          case 0:
            return;
          case 1:
            try {
              this.wait();
            } catch (InterruptedException var6) {
            }
            break;
          case 2:
            this.terminate();
          case 3:
            try {
              this.child.exitValue();
            } catch (IllegalThreadStateException var7) {
              long var2 = System.currentTimeMillis();
              if (this.waitTime > var2) {
                try {
                  this.wait(this.waitTime - var2);
                } catch (InterruptedException var5) {
                }
                continue;
              }
            }

            this.childGone();
            return;
        }
      }
    }

    void shutdownFast() {
      Process var1 = this.child;
      if (var1 != null) {
        var1.destroy();
      }

    }

    synchronized void shutdown() {
      this.reset();
      this.terminate();
      this.await();
    }

    MarshalledObject<? extends Remote> activate(ActivationID var1, boolean var2) throws ActivationException {
      Object var3 = null;

      for(int var4 = 2; var4 > 0; --var4) {
        ActivationInstantiator var5;
        long var6;
        Activation.ObjectEntry var8;
        synchronized(this) {
          var8 = this.getObjectEntry(var1);
          if (!var2 && var8.stub != null) {
            return var8.stub;
          }

          var5 = this.getInstantiator(this.groupID);
          var6 = this.incarnation;
        }

        boolean var9 = false;
        boolean var10 = false;

        try {
          return var8.activate(var1, var2, var5);
        } catch (NoSuchObjectException var13) {
          var9 = true;
          var3 = var13;
        } catch (ConnectException var14) {
          var9 = true;
          var10 = true;
          var3 = var14;
        } catch (ConnectIOException var15) {
          var9 = true;
          var10 = true;
          var3 = var15;
        } catch (InactiveGroupException var16) {
          var9 = true;
          var3 = var16;
        } catch (RemoteException var17) {
          if (var3 == null) {
            var3 = var17;
          }
        }

        if (var9) {
          try {
            System.err.println(MessageFormat.format(Activation.getTextResource("rmid.group.inactive"), ((Exception)var3).toString()));
            ((Exception)var3).printStackTrace();
            Activation.this.getGroupEntry(this.groupID).inactiveGroup(var6, var10);
          } catch (UnknownGroupException var12) {
          }
        }
      }

      throw new ActivationException("object activation failed after 2 tries", (Throwable)var3);
    }

    private ActivationInstantiator getInstantiator(ActivationGroupID var1) throws ActivationException {
      assert Thread.holdsLock(this);

      this.await();
      if (this.group != null) {
        return this.group;
      } else {
        this.checkRemoved();
        boolean var2 = false;

        try {
          this.groupName = Activation.this.Pstartgroup();
          var2 = true;
          String[] var3 = Activation.this.activationArgs(this.desc);
          Activation.this.checkArgs(this.desc, var3);
          if (Activation.debugExec) {
            StringBuffer var4 = new StringBuffer(var3[0]);

            for(int var5 = 1; var5 < var3.length; ++var5) {
              var4.append(' ');
              var4.append(var3[var5]);
            }

            System.err.println(MessageFormat.format(Activation.getTextResource("rmid.exec.command"), var4.toString()));
          }

          try {
            this.child = Runtime.getRuntime().exec(var3);
            this.status = 1;
            ++this.incarnation;
            this.watchdog = new Activation.GroupEntry.Watchdog();
            this.watchdog.start();
            Activation.this.addLogRecord(new Activation.LogGroupIncarnation(var1, this.incarnation));
            PipeWriter.plugTogetherPair(this.child.getInputStream(), System.out, this.child.getErrorStream(), System.err);
            MarshalOutputStream var30 = new MarshalOutputStream(this.child.getOutputStream());
            Throwable var32 = null;

            try {
              var30.writeObject(var1);
              var30.writeObject(this.desc);
              var30.writeLong(this.incarnation);
              var30.flush();
            } catch (Throwable var25) {
              var32 = var25;
              throw var25;
            } finally {
              if (var30 != null) {
                if (var32 != null) {
                  try {
                    var30.close();
                  } catch (Throwable var24) {
                    var32.addSuppressed(var24);
                  }
                } else {
                  var30.close();
                }
              }

            }
          } catch (IOException var27) {
            this.terminate();
            throw new ActivationException("unable to create activation group", var27);
          }

          try {
            long var31 = System.currentTimeMillis();
            long var6 = var31 + Activation.execTimeout;

            do {
              this.wait(var6 - var31);
              if (this.group != null) {
                ActivationInstantiator var8 = this.group;
                return var8;
              }

              var31 = System.currentTimeMillis();
            } while(this.status == 1 && var31 < var6);
          } catch (InterruptedException var28) {
          }

          this.terminate();
          throw new ActivationException(this.removed ? "activation group unregistered" : "timeout creating child process");
        } finally {
          if (var2) {
            Activation.this.Vstartgroup();
          }

        }
      }
    }

    private class Watchdog extends Thread {
      private final Process groupProcess;
      private final long groupIncarnation;
      private boolean canInterrupt;
      private boolean shouldQuit;
      private boolean shouldRestart;

      Watchdog() {
        super("WatchDog-" + GroupEntry.this.groupName + "-" + GroupEntry.this.incarnation);
        this.groupProcess = GroupEntry.this.child;
        this.groupIncarnation = GroupEntry.this.incarnation;
        this.canInterrupt = true;
        this.shouldQuit = false;
        this.shouldRestart = true;
        this.setDaemon(true);
      }

      public void run() {
        if (!this.shouldQuit) {
          try {
            this.groupProcess.waitFor();
          } catch (InterruptedException var4) {
            return;
          }

          boolean var1 = false;
          synchronized(GroupEntry.this) {
            if (this.shouldQuit) {
              return;
            }

            this.canInterrupt = false;
            interrupted();
            if (this.groupIncarnation == GroupEntry.this.incarnation) {
              var1 = this.shouldRestart && !Activation.this.shuttingDown;
              GroupEntry.this.reset();
              GroupEntry.this.childGone();
            }
          }

          if (var1) {
            GroupEntry.this.restartServices();
          }

        }
      }

      void dispose() {
        this.shouldQuit = true;
        if (this.canInterrupt) {
          this.interrupt();
        }

      }

      void noRestart() {
        this.shouldRestart = false;
      }
    }
  }

  private class ShutdownHook extends Thread {
    ShutdownHook() {
      super("rmid ShutdownHook");
    }

    public void run() {
      synchronized(Activation.this) {
        Activation.this.shuttingDown = true;
      }

      Iterator var1 = Activation.this.groupTable.values().iterator();

      while(var1.hasNext()) {
        Activation.GroupEntry var2 = (Activation.GroupEntry)var1.next();
        var2.shutdownFast();
      }

    }
  }

  private class Shutdown extends Thread {
    Shutdown() {
      super("rmid Shutdown");
    }

    public void run() {
      try {
        Activation.unexport(Activation.this.activator);
        Activation.unexport(Activation.this.system);
        Iterator var1 = Activation.this.groupTable.values().iterator();

        while(true) {
          if (!var1.hasNext()) {
            Runtime.getRuntime().removeShutdownHook(Activation.this.shutdownHook);
            Activation.unexport(Activation.this.monitor);

            try {
              synchronized(Activation.this.log) {
                Activation.this.log.close();
              }
            } catch (IOException var9) {
            }
            break;
          }

          Activation.GroupEntry var2 = (Activation.GroupEntry)var1.next();
          var2.shutdown();
        }
      } finally {
        System.err.println(Activation.getTextResource("rmid.daemon.shutdown"));
        System.exit(0);
      }

    }
  }

  class ActivationSystemImpl extends RemoteServer implements ActivationSystem {
    private static final long serialVersionUID = 9100152600327688967L;

    ActivationSystemImpl(int var2, RMIServerSocketFactory var3) throws RemoteException {
      LiveRef var4 = new LiveRef(new ObjID(4), var2, (RMIClientSocketFactory)null, var3);
      Activation.SameHostOnlyServerRef var5 = new Activation.SameHostOnlyServerRef(var4, "ActivationSystem.nonLocalAccess");
      this.ref = var5;
      var5.exportObject(this, (Object)null);
    }

    public ActivationID registerObject(ActivationDesc var1) throws ActivationException, UnknownGroupException, RemoteException {
      Activation.this.checkShutdown();
      ActivationGroupID var2 = var1.getGroupID();
      ActivationID var3 = new ActivationID(Activation.this.activatorStub);
      Activation.this.getGroupEntry(var2).registerObject(var3, var1, true);
      return var3;
    }

    public void unregisterObject(ActivationID var1) throws ActivationException, UnknownObjectException, RemoteException {
      Activation.this.checkShutdown();
      Activation.this.getGroupEntry(var1).unregisterObject(var1, true);
    }

    public ActivationGroupID registerGroup(ActivationGroupDesc var1) throws ActivationException, RemoteException {
      Thread.dumpStack();
      Activation.this.checkShutdown();
      Activation.this.checkArgs(var1, (String[])null);
      ActivationGroupID var2 = new ActivationGroupID(Activation.this.systemStub);
      Activation.GroupEntry var3 = Activation.this.new GroupEntry(var2, var1);
      Activation.this.groupTable.put(var2, var3);
      Activation.this.addLogRecord(new Activation.LogRegisterGroup(var2, var1));
      return var2;
    }

    public ActivationMonitor activeGroup(ActivationGroupID var1, ActivationInstantiator var2, long var3) throws ActivationException, UnknownGroupException, RemoteException {
      Activation.this.checkShutdown();
      Activation.this.getGroupEntry(var1).activeGroup(var2, var3);
      return Activation.this.monitor;
    }

    public void unregisterGroup(ActivationGroupID var1) throws ActivationException, UnknownGroupException, RemoteException {
      Activation.this.checkShutdown();
      Activation.this.removeGroupEntry(var1).unregisterGroup(true);
    }

    public ActivationDesc setActivationDesc(ActivationID var1, ActivationDesc var2) throws ActivationException, UnknownObjectException, RemoteException {
      Activation.this.checkShutdown();
      if (!Activation.this.getGroupID(var1).equals(var2.getGroupID())) {
        throw new ActivationException("ActivationDesc contains wrong group");
      } else {
        return Activation.this.getGroupEntry(var1).setActivationDesc(var1, var2, true);
      }
    }

    public ActivationGroupDesc setActivationGroupDesc(ActivationGroupID var1, ActivationGroupDesc var2) throws ActivationException, UnknownGroupException, RemoteException {
      Activation.this.checkShutdown();
      Activation.this.checkArgs(var2, (String[])null);
      return Activation.this.getGroupEntry(var1).setActivationGroupDesc(var1, var2, true);
    }

    public ActivationDesc getActivationDesc(ActivationID var1) throws ActivationException, UnknownObjectException, RemoteException {
      Activation.this.checkShutdown();
      return Activation.this.getGroupEntry(var1).getActivationDesc(var1);
    }

    public ActivationGroupDesc getActivationGroupDesc(ActivationGroupID var1) throws ActivationException, UnknownGroupException, RemoteException {
      Activation.this.checkShutdown();
      return Activation.this.getGroupEntry(var1).desc;
    }

    public void shutdown() throws AccessException {
      Object var1 = Activation.this.startupLock;
      if (var1 != null) {
        synchronized(var1) {
          ;
        }
      }

      synchronized(Activation.this) {
        if (!Activation.this.shuttingDown) {
          Activation.this.shuttingDown = true;
          (Activation.this.new Shutdown()).start();
        }

      }
    }
  }

  static class SameHostOnlyServerRef extends UnicastServerRef {
    private static final long serialVersionUID = 1234L;
    private String accessKind;

    SameHostOnlyServerRef(LiveRef var1, String var2) {
      super(var1);
      this.accessKind = var2;
    }

    protected void unmarshalCustomCallData(ObjectInput var1) throws IOException, ClassNotFoundException {
      RegistryImpl.checkAccess(this.accessKind);
      super.unmarshalCustomCallData(var1);
    }
  }

  class ActivationMonitorImpl extends UnicastRemoteObject implements ActivationMonitor {
    private static final long serialVersionUID = -6214940464757948867L;

    ActivationMonitorImpl(int var2, RMIServerSocketFactory var3) throws RemoteException {
      super(var2, (RMIClientSocketFactory)null, var3);
    }

    public void inactiveObject(ActivationID var1) throws UnknownObjectException, RemoteException {
      try {
        Activation.this.checkShutdown();
      } catch (ActivationException var3) {
        return;
      }

      RegistryImpl.checkAccess("Activator.inactiveObject");
      Activation.this.getGroupEntry(var1).inactiveObject(var1);
    }

    public void activeObject(ActivationID var1, MarshalledObject<? extends Remote> var2) throws UnknownObjectException, RemoteException {
      try {
        Activation.this.checkShutdown();
      } catch (ActivationException var4) {
        return;
      }

      RegistryImpl.checkAccess("ActivationSystem.activeObject");
      Activation.this.getGroupEntry(var1).activeObject(var1, var2);
    }

    public void inactiveGroup(ActivationGroupID var1, long var2) throws UnknownGroupException, RemoteException {
      try {
        Activation.this.checkShutdown();
      } catch (ActivationException var5) {
        return;
      }

      RegistryImpl.checkAccess("ActivationMonitor.inactiveGroup");
      Activation.this.getGroupEntry(var1).inactiveGroup(var2, false);
    }
  }

  class ActivatorImpl extends RemoteServer implements Activator {
    private static final long serialVersionUID = -3654244726254566136L;

    ActivatorImpl(int var2, RMIServerSocketFactory var3) throws RemoteException {
      LiveRef var4 = new LiveRef(new ObjID(1), var2, (RMIClientSocketFactory)null, var3);
      UnicastServerRef var5 = new UnicastServerRef(var4);
      this.ref = var5;
      var5.exportObject(this, (Object)null, false);
    }

    public MarshalledObject<? extends Remote> activate(ActivationID var1, boolean var2) throws ActivationException, UnknownObjectException, RemoteException {
      Activation.this.checkShutdown();
      return Activation.this.getGroupEntry(var1).activate(var1, var2);
    }
  }

  private static class SystemRegistryImpl extends RegistryImpl {
    private static final String NAME = ActivationSystem.class.getName();
    private static final long serialVersionUID = 4877330021609408794L;
    private final ActivationSystem systemStub;

    SystemRegistryImpl(int var1, RMIClientSocketFactory var2, RMIServerSocketFactory var3, ActivationSystem var4) throws RemoteException {
      super(var1, var2, var3);
      this.systemStub = var4;
    }

    public Remote lookup(String var1) throws RemoteException, NotBoundException {
      return (Remote)(var1.equals(NAME) ? this.systemStub : super.lookup(var1));
    }

    public String[] list() throws RemoteException {
      String[] var1 = super.list();
      int var2 = var1.length;
      String[] var3 = new String[var2 + 1];
      if (var2 > 0) {
        System.arraycopy(var1, 0, var3, 0, var2);
      }

      var3[var2] = NAME;
      return var3;
    }

    public void bind(String var1, Remote var2) throws RemoteException, AlreadyBoundException, AccessException {
      if (var1.equals(NAME)) {
        throw new AccessException("binding ActivationSystem is disallowed");
      } else {
        RegistryImpl.checkAccess("ActivationSystem.bind");
        super.bind(var1, var2);
      }
    }

    public void unbind(String var1) throws RemoteException, NotBoundException, AccessException {
      if (var1.equals(NAME)) {
        throw new AccessException("unbinding ActivationSystem is disallowed");
      } else {
        RegistryImpl.checkAccess("ActivationSystem.unbind");
        super.unbind(var1);
      }
    }

    public void rebind(String var1, Remote var2) throws RemoteException, AccessException {
      if (var1.equals(NAME)) {
        throw new AccessException("binding ActivationSystem is disallowed");
      } else {
        RegistryImpl.checkAccess("ActivationSystem.rebind");
        super.rebind(var1, var2);
      }
    }
  }
}
