### SML

![Build](https://github.com/Stoozy/SML/actions/workflows/rust.yml/badge.svg)
[![License: GPL v2](https://img.shields.io/badge/License-GPL%20v2-blue.svg)](https://www.gnu.org/licenses/old-licenses/gpl-2.0.en.html)

Welcome to stoozys minecraft launcher!

## About

This is a CLI program that allows you to install and launch curseforge modpacks. Currently, it supports only vanilla and forge instances. Fabric will be supported soon. 

## Motivation

I am working on this project purely for the learning experience, there are working minecraft modded launchers out there already, such as MultiMC. However, this project should benefit those who don't want a fully featured GUI program and would rather do things in a shell.

## Installation

There are some plans to add it to some popular package managers, but this should do for now.

Windows:
 - Go to releases
 - Install using the msi installer

Linux:
 - Go to releases
 - Download and extract the tarball
 - Run the install script as root


## Usage


```
SML 0.1.1
Stoozy 
A Minecraft Modded Launcher Command Line Interface

USAGE:
    sml.exe [FLAGS] [OPTIONS]

FLAGS:
        --auth       Log in through mojang
    -h, --help       Prints help information
        --list       Lists all SML instances
    -V, --version    Prints version information

OPTIONS:
    -a, --add-instance <TYPE>    Add a new instance. Types can be the following : forge, vanilla, or fabric.
    -c, --config <ID>            Configures instance with the ID provided
        --launch <ID>            Launches instance with specific ID
        --print-config <ID>      Shows the custom flags for an instance
    -r, --remove <ID>            Removes instance with the ID provided
        --rename <ID>            Rename the instance with provided ID
```



## What doesn't work right now
 - Fabric Modpacks

## Issues

If you encounter any issues, feel free to open an [issue](https://github.com/Stoozy/SML/issues) on github

