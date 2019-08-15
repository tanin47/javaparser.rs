package sun.rmi.transport.tcp;

import java.security.AccessController;
import java.security.PrivilegedAction;
import java.util.ArrayList;
import java.util.List;
import sun.rmi.runtime.NewThreadAction;
import sun.rmi.transport.Connection;

class ConnectionAcceptor implements Runnable {
  private TCPTransport transport;
  private List<Connection> queue = new ArrayList();
  private static int threadNum = 0;

  public ConnectionAcceptor(TCPTransport var1) {
    this.transport = var1;
  }

  public void startNewAcceptor() {
    Thread var1 = (Thread)AccessController.doPrivileged((PrivilegedAction)(new NewThreadAction(this, "Multiplex Accept-" + ++threadNum, true)));
    var1.start();
  }

  public void accept(Connection var1) {
    synchronized(this.queue) {
      this.queue.add(var1);
      this.queue.notify();
    }
  }

  public void run() {
    Connection var1;
    synchronized(this.queue) {
      while(this.queue.size() == 0) {
        try {
          this.queue.wait();
        } catch (InterruptedException var5) {
        }
      }

      this.startNewAcceptor();
      var1 = (Connection)this.queue.remove(0);
    }

    this.transport.handleMessages(var1, true);
  }
}
