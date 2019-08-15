package sun.misc;

import java.lang.ref.PhantomReference;
import java.lang.ref.ReferenceQueue;
import java.security.AccessController;
import java.security.PrivilegedAction;

public class Cleaner extends PhantomReference<Object> {
  private static final ReferenceQueue<Object> dummyQueue = new ReferenceQueue();
  private static Cleaner first = null;
  private Cleaner next = null;
  private Cleaner prev = null;
  private final Runnable thunk;

  private static synchronized Cleaner add(Cleaner var0) {
    if (first != null) {
      var0.next = first;
      first.prev = var0;
    }

    first = var0;
    return var0;
  }

  private static synchronized boolean remove(Cleaner var0) {
    if (var0.next == var0) {
      return false;
    } else {
      if (first == var0) {
        if (var0.next != null) {
          first = var0.next;
        } else {
          first = var0.prev;
        }
      }

      if (var0.next != null) {
        var0.next.prev = var0.prev;
      }

      if (var0.prev != null) {
        var0.prev.next = var0.next;
      }

      var0.next = var0;
      var0.prev = var0;
      return true;
    }
  }

  private Cleaner(Object var1, Runnable var2) {
    super(var1, dummyQueue);
    this.thunk = var2;
  }

  public static Cleaner create(Object var0, Runnable var1) {
    return var1 == null ? null : add(new Cleaner(var0, var1));
  }

  public void clean() {
    if (remove(this)) {
      try {
        this.thunk.run();
      } catch (final Throwable var2) {
        AccessController.doPrivileged(new PrivilegedAction<Void>() {
          public Void run() {
            if (System.err != null) {
              (new Error("Cleaner terminated abnormally", var2)).printStackTrace();
            }

            System.exit(1);
            return null;
          }
        });
      }

    }
  }
}
