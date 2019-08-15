package javax.smartcardio;

import java.security.AccessController;
import java.security.NoSuchAlgorithmException;
import java.security.NoSuchProviderException;
import java.security.PrivilegedAction;
import java.security.Provider;
import java.security.Security;
import java.util.Collections;
import java.util.List;
import sun.security.action.GetPropertyAction;
import sun.security.jca.GetInstance;

public final class TerminalFactory {
  private static final String PROP_NAME = "javax.smartcardio.TerminalFactory.DefaultType";
  private static final String defaultType;
  private static final TerminalFactory defaultFactory;
  private final TerminalFactorySpi spi;
  private final Provider provider;
  private final String type;

  private TerminalFactory(TerminalFactorySpi var1, Provider var2, String var3) {
    this.spi = var1;
    this.provider = var2;
    this.type = var3;
  }

  public static String getDefaultType() {
    return defaultType;
  }

  public static TerminalFactory getDefault() {
    return defaultFactory;
  }

  public static TerminalFactory getInstance(String var0, Object var1) throws NoSuchAlgorithmException {
    GetInstance.Instance var2 = GetInstance.getInstance("TerminalFactory", TerminalFactorySpi.class, var0, var1);
    return new TerminalFactory((TerminalFactorySpi)var2.impl, var2.provider, var0);
  }

  public static TerminalFactory getInstance(String var0, Object var1, String var2) throws NoSuchAlgorithmException, NoSuchProviderException {
    GetInstance.Instance var3 = GetInstance.getInstance("TerminalFactory", TerminalFactorySpi.class, var0, var1, var2);
    return new TerminalFactory((TerminalFactorySpi)var3.impl, var3.provider, var0);
  }

  public static TerminalFactory getInstance(String var0, Object var1, Provider var2) throws NoSuchAlgorithmException {
    GetInstance.Instance var3 = GetInstance.getInstance("TerminalFactory", TerminalFactorySpi.class, var0, var1, var2);
    return new TerminalFactory((TerminalFactorySpi)var3.impl, var3.provider, var0);
  }

  public Provider getProvider() {
    return this.provider;
  }

  public String getType() {
    return this.type;
  }

  public CardTerminals terminals() {
    return this.spi.engineTerminals();
  }

  public String toString() {
    return "TerminalFactory for type " + this.type + " from provider " + this.provider.getName();
  }

  static {
    String var0 = ((String)AccessController.doPrivileged((PrivilegedAction)(new GetPropertyAction("javax.smartcardio.TerminalFactory.DefaultType", "PC/SC")))).trim();
    TerminalFactory var1 = null;

    try {
      var1 = getInstance(var0, (Object)null);
    } catch (Exception var5) {
    }

    if (var1 == null) {
      try {
        var0 = "PC/SC";
        Provider var2 = Security.getProvider("SunPCSC");
        if (var2 == null) {
          Class var3 = Class.forName("sun.security.smartcardio.SunPCSC");
          var2 = (Provider)var3.newInstance();
        }

        var1 = getInstance(var0, (Object)null, (Provider)var2);
      } catch (Exception var4) {
      }
    }

    if (var1 == null) {
      var0 = "None";
      var1 = new TerminalFactory(TerminalFactory.NoneFactorySpi.INSTANCE, TerminalFactory.NoneProvider.INSTANCE, "None");
    }

    defaultType = var0;
    defaultFactory = var1;
  }

  private static final class NoneCardTerminals extends CardTerminals {
    static final CardTerminals INSTANCE = new TerminalFactory.NoneCardTerminals();

    public List<CardTerminal> list(CardTerminals.State var1) throws CardException {
      if (var1 == null) {
        throw new NullPointerException();
      } else {
        return Collections.emptyList();
      }
    }

    public boolean waitForChange(long var1) throws CardException {
      throw new IllegalStateException("no terminals");
    }
  }

  private static final class NoneFactorySpi extends TerminalFactorySpi {
    static final TerminalFactorySpi INSTANCE = new TerminalFactory.NoneFactorySpi();

    protected CardTerminals engineTerminals() {
      return TerminalFactory.NoneCardTerminals.INSTANCE;
    }
  }

  private static final class NoneProvider extends Provider {
    private static final long serialVersionUID = 2745808869881593918L;
    static final Provider INSTANCE = new TerminalFactory.NoneProvider();

    private NoneProvider() {
      super("None", 1.0D, "none");
    }
  }
}
