# RSCube

A Minecraft Classic Server written in Rust. It is a multi-threaded server built on an MPSC architectuure.

# Features

* Base MC Classic Protocol
* Multi-Threading in a Multi Producer-Single Consumer architecture.
* Initial CPE Support (Handshake and TwoWayPing, CustomBlocks, FastMap)
* World saving with /save.
* World generation using supplied python script (world_generate.py, must install opensimplex as dependency) supporting two types:  
  * Flat World 
  * Simple Procedural World (using Simplex Noise generation)

* Uses MCSharp World Format
* Some primitive commands (/kickself, /op, /tp, /save)
# TODO

* Level generator
* Multiple maps
* More CPE extensions
