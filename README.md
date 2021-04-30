### SML

![Build](https://github.com/Stoozy/SML/actions/workflows/rust.yml/badge.svg)

[![License: GPL v2](https://img.shields.io/badge/License-GPL%20v2-blue.svg)](https://www.gnu.org/licenses/old-licenses/gpl-2.0.en.html)

Welcome to stoozys minecraft modded launcher!

## About

This is a CLI program that allows you to install and launch modpacks. Currently it only supports forge only but soon it will have fabric and vanilla support.

## Installation

Currently, there are no releases, therefore you must install via building the source code. To build the source code, you must have rust installed along with cargo. Then run the following command in the cloned github repository directory `cargo build`.

Then move the binary from `../SML/target/debug/sml` to somewhere else. **Note**: If you're on windows, put it somewhere with a small path length. Somewhere like `C:\SML\sml`. If you don't do this, the program may not work properly due to the long invocation (There's a limit to how long the invocation can be, see https://docs.microsoft.com/en-us/troubleshoot/windows-client/shell-experience/command-line-string-limitation). In linux, this is not an issue. After that, you just need to add the `sml` binaries directory to `PATH`. From there run `sml -h` on the command line to get started using the app.

## Forge Installer

This launcher uses a wrapper class around the forge installer in order to automate the installation. See  https://github.com/xfl03/ForgeInstallerHeadless



