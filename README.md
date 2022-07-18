# EZAIMD
Command line tool that allows the user to easly run ab initio molecular dynamics (AIMD) interfacing with the Gaussian16 quantum chemical package.

# Configuration
EZAIMD requires the use of a configuration file, which must be named 'config.yaml', in the root directory of a simulation. This configuration is used to generate input for the quantum chemical package [Gaussian16](https://gaussian.com/gaussian16/). An example configuration is given bellow:

config.yaml:
```yaml
---
mem: "140GB"
cpu: "0-47"
gpu: ~
checkpoint: "output.chk"
key_words: "#p WB97XD/Def2tzvpp SCF=XQC"
title: "single point"
charge: 0
multiplicity: 1
```
