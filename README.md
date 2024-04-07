# Plot train timetable
Rust program to retrieve and plot timetables of daily circulating trains on the Savona-Ventimiglia railway line.
It retrieves circulating train numbers from https://trainstats.altervista.org (only Trenitalia-operated passenger trains, not 100% accurate, some InterCity trains are missing) and then fetches data from Trenitalia's Viaggiatreno.

Currently station data is hardcoded, and only works with the Savona-Ventimiglia railway line, but in future it is possible that I will make it work for every section of Italian railways.
