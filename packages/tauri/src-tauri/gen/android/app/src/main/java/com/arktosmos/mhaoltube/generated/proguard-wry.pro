# THIS FILE IS AUTO-GENERATED. DO NOT MODIFY!!

# Copyright 2020-2023 Tauri Programme within The Commons Conservancy
# SPDX-License-Identifier: Apache-2.0
# SPDX-License-Identifier: MIT

-keep class com.arktosmos.mhaoltube.* {
  native <methods>;
}

-keep class com.arktosmos.mhaoltube.WryActivity {
  public <init>(...);

  void setWebView(com.arktosmos.mhaoltube.RustWebView);
  java.lang.Class getAppClass(...);
  java.lang.String getVersion();
}

-keep class com.arktosmos.mhaoltube.Ipc {
  public <init>(...);

  @android.webkit.JavascriptInterface public <methods>;
}

-keep class com.arktosmos.mhaoltube.RustWebView {
  public <init>(...);

  void loadUrlMainThread(...);
  void loadHTMLMainThread(...);
  void evalScript(...);
}

-keep class com.arktosmos.mhaoltube.RustWebChromeClient,com.arktosmos.mhaoltube.RustWebViewClient {
  public <init>(...);
}
