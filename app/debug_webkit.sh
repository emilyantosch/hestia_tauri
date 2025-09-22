#!/bin/bash

# WebKit GTK debugging script for negative proportions issue

echo "=== WebKit GTK Debug Session ==="

# Enable webkit debugging
export WEBKIT_DEBUG=1
export G_MESSAGES_DEBUG=all
export GSK_DEBUG=opengl
export GDK_DEBUG=events
export GTK_DEBUG=interactive

# WebKit specific debugging
export WEBKIT_DISABLE_COMPOSITING_MODE=1
export WEBKIT_FORCE_SANDBOX=0

# WebKit scaling fixes
export GDK_SCALE=1
export GDK_DPI_SCALE=1
export WEBKIT_DISABLE_AUTO_DPI=1
export QT_AUTO_SCREEN_SCALE_FACTOR=0
export QT_SCALE_FACTOR=1
export WEBKIT_FORCE_DEVICE_SCALE_FACTOR=1.0

# GTK font scaling fixes
export GTK_FONT_SCALE=1.0
export GDK_FONT_SCALE=1.0
export WEBKIT_FONT_SCALE=1.0

# Additional webkit flags
export WEBKIT_INSPECTOR_SERVER=127.0.0.1:9222

echo "Environment variables set:"
echo "WEBKIT_DEBUG=$WEBKIT_DEBUG"
echo "G_MESSAGES_DEBUG=$G_MESSAGES_DEBUG"
echo "GSK_DEBUG=$GSK_DEBUG"
echo "GDK_DEBUG=$GDK_DEBUG"
echo "GDK_SCALE=$GDK_SCALE"
echo "GDK_DPI_SCALE=$GDK_DPI_SCALE"
echo "WEBKIT_FORCE_DEVICE_SCALE_FACTOR=$WEBKIT_FORCE_DEVICE_SCALE_FACTOR"

echo "Starting Tauri with WebKit debugging..."
cargo tauri dev 2>&1 | tee webkit_debug.log

echo "Debug log saved to webkit_debug.log"