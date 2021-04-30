### SML

![Build](https://github.com/Stoozy/SML/actions/workflows/rust.yml/badge.svg)
[![License: GPL v2](https://img.shields.io/badge/License-GPL%20v2-blue.svg)](https://www.gnu.org/licenses/old-licenses/gpl-2.0.en.html)

Welcome to stoozys minecraft launcher!

## About

This is a CLI program that allows you to install and launch curseforge modpacks. Currently it only supports forge only but soon it will have fabric and vanilla support.

## Motivation

I am working on this project purely for the learning experience, there are working minecraft modded launchers out there already, such as MultiMC. However, this project should benefit those who don't want a fully featured GUI program and would rather do things in a shell.


## Usage

```
SML 0.1.0
Stoozy <mahinsemail@gmail.com>
A Minecraft Modded Launcher Command Line Interface

USAGE:
    sml [FLAGS] [OPTIONS]

FLAGS:
    -a, --auth       Log in through mojang
    -c, --config     configures instance
    -h, --help       Prints help information
        --list       Lists all SML instances
    -V, --version    Prints version information

OPTIONS:
    -i, --install <ID>    Searches for project in curseforge with given ID and installs it
        --launch <ID>     Launches instance with specific ID
    -r <ID>               Removes instance with the ID provided
```

## Forge Installer

This launcher uses a wrapper class around the forge installer written by @xfl03 in order to automate the installation. See https://github.com/xfl03/ForgeInstallerHeadless

## What doesn't work right now

Fabric and Vanilla are not supported yet. 
Forge packs with version less than 1.13.


## Issues

If you encounter any issues, feel free to open an [issue](https://github.com/Stoozy/SML/issues) on github

