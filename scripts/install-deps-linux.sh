#!/bin/bash
set -e

echo "正在检测操作系统包管理器..."

if command -v dnf &> /dev/null; then
    echo "检测到 DNF 包管理器 (Fedora/RHEL)..."
    sudo dnf check-update || true
    sudo dnf install -y \
        pkgconf-pkg-config \
        fontconfig-devel \
        libX11-devel \
        libxcb-devel \
        libxkbcommon-devel \
        libxkbcommon-x11-devel \
        wayland-devel \
        vulkan-loader-devel \
        openssl-devel \
        dbus-devel \
        alsa-lib-devel \
        cmake \
        gcc-c++
elif command -v apt-get &> /dev/null; then
    echo "检测到 APT 包管理器 (Ubuntu/Debian)..."
    sudo apt-get update
    sudo apt-get install -y \
        pkg-config \
        libfontconfig1-dev \
        libx11-dev \
        libxcb-shape0-dev \
        libxcb-xfixes0-dev \
        libxkbcommon-dev \
        libxkbcommon-x11-dev \
        libwayland-dev \
        libvulkan-dev \
        libssl-dev \
        libdbus-1-dev \
        libasound2-dev \
        cmake \
        build-essential
else
    echo "未识别的包管理器，请手动安装相关开发库 (fontconfig, x11, wayland, vulkan, xkbcommon-x11)。"
    exit 1
fi

echo "--------------------------------------------------"
echo "依赖安装完成！现在你可以尝试重新运行: cargo run"
