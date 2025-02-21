//
//  Generated file. Do not edit.
//

// clang-format off

#include "generated_plugin_registrant.h"

#include <desktop_window/desktop_window_plugin.h>
#include <hid4flutter/hid4flutter_plugin.h>

void fl_register_plugins(FlPluginRegistry* registry) {
  g_autoptr(FlPluginRegistrar) desktop_window_registrar =
      fl_plugin_registry_get_registrar_for_plugin(registry, "DesktopWindowPlugin");
  desktop_window_plugin_register_with_registrar(desktop_window_registrar);
  g_autoptr(FlPluginRegistrar) hid4flutter_registrar =
      fl_plugin_registry_get_registrar_for_plugin(registry, "Hid4flutterPlugin");
  hid4flutter_plugin_register_with_registrar(hid4flutter_registrar);
}
