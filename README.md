# EZAIMD
Command line tool that allows the user to easly run ab initio molecular dynamics (AIMD) interfacing with the Gaussian16 quantum chemical package.

# Configuration
EZAIMD requires the use of a configuration file, which must be named `config.yaml`, in the root directory of a simulation. This configuration is used to generate input for the quantum chemical package [Gaussian16](https://gaussian.com/gaussian16/). An example configuration is given bellow:

config.yaml:
```yaml
---
mem: "140GB"
cpu: "0-47"
gpu: ~
checkpoint: "output.chk"
key_words: "#p WB97XD/Def2tzvpp SCF=XQC force"
title: "single point"
charge: 0
multiplicity: 1
```
NOTE: For a successful simulation, the `force` keyword is REQUIRED!

# Setting Up A Simulation
A simulation requires two items, the configuration and a valid gaussian16 output file. Where a valid output file will contain molecular coordinates in standard orientation. If multiply coordinates are valid in an output file, the last set of coordinates will be used. EZAIMD will through an error when no atomic information can be read.

Once the requirements have been met, one can start a default settings simulation with the following command:

`EZAIMD [Gaussian16 outputfile]`

This will begin a simulation with:

Time step: 1fs

Number of steps: 10000

# Options
A number of options are available to the user.

`--freeze`: freeze the requested atoms during the simulaiton

## Example
`--freeze 1-10,90-100` will freeze atoms 1-10 and 90-100.
