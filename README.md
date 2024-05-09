# Infotreno-cli
Rust command line program to get Trenitalia's train information from the Viaggiatreno API.

## Usage
execute the program with commands:
- `track`: track a train by its train number
- `station`: get information about a station by entering its name or code
- `news`: print "Notizie Infomobilità" (trenitalia news about disruptions and delays)
- `help`: get full list of commands and options

A full documentation of the commands and options can also be found by running `infotreno-cli` without specifying any command.

## Roadmap
- Print train delay history for a certain train at a certain station
- Print currently circulating trains between two sections of main lines (filtering by train type is possible)
- Print all currently circulating long distance trains
- Print currently circulating trains between two sections on a branch regional line
- Plot currently circulating trains between two sections

Existing iaggiatreno API documentation: 
- https://github.com/roughconsensusandrunningcode/TrainMonitor/wiki/API-del-sistema-Viaggiatreno
- https://github.com/sabas/trenitalia.

## Build 
To build the program, Rust and Cargo are required.