package com.sun.xml.internal.ws.server;

import com.sun.org.glassfish.gmbal.Description;
import com.sun.org.glassfish.gmbal.InheritedAttribute;
import com.sun.org.glassfish.gmbal.InheritedAttributes;
import com.sun.org.glassfish.gmbal.ManagedData;

@ManagedData
@Description("WebServiceFeature")
@InheritedAttributes({@InheritedAttribute(
  methodName = "getID",
  description = "unique id for this feature"
), @InheritedAttribute(
  methodName = "isEnabled",
  description = "true if this feature is enabled"
)})
interface DummyWebServiceFeature {
}
