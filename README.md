# Plot train timetable
Rust program to retrieve and plot timetables of daily circulating trains on the Savona-Ventimiglia railway line.
It retrieves circulating train numbers from https://trainstats.altervista.org (only Trenitalia-operated passenger trains, not 100% accurate, some InterCity trains are missing) and then fetches data from Trenitalia's Viaggiatreno.

Currently station data is hardcoded, and only works with the Savona-Ventimiglia railway line.

Viaggiatreno API documentation: 
- https://github.com/roughconsensusandrunningcode/TrainMonitor/wiki/API-del-sistema-Viaggiatreno
- https://github.com/sabas/trenitalia.

## Roadmap
- Plot currently circulating trains between two sections
- Print currently circulating trains between two sections of main lines (filtering by train type is possible)
- Print all currently circulating long distance trains
- Print currently circulating trains between two sections on a branch regional line
- Print arriving and departing trains at a certain station
- Print train punctuality informations
- Print train delay history for a certain train at a certain station
- Format and print _Notizie infomobilità_
