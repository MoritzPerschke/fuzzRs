# FuzzRs
A fuzzing tool written in rust, which provides an easy to use UI to avoid having to edit and re-edit the same terminal command 

## Description
This project was started out of boredom during my summer holidays after starting the HackTheBox bug bounty hunter path.
I found myself using ffuf a lot, but I found it cumbersome to edit parameters in a terminal command.
I also wanted to get more used to writing Rust, so I started building a fuzzer that utilizes ratatui to provide a terminal UI with keybinds to jump around to the different parameters.

## Current state
Right now the fuzzer is barely in a usable state, as it only provides the usage of GET requests and can only fuzz an URL parameter.

## Goals
The main goal is to have built a fuzzer that I would use over any alternative.

Next steps:
- [ ] implement different types of requests
- [ ] implement fuzzing of data and request headers
- [ ] implement the capability to set a 'wordlist directory', to avoid having to type the full wordlist path
