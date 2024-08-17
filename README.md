## Tracked

Tracked is a part of https://simwatch.vatsimnerd.com and is designed to store VATSIM pilot's tracks in compact binary format.

The update API is designed to receive bulk update requests and able to update multiple tracks at once. Tracked does not store equivalent points so stationary aircraft only take 2 points to describe the entire period of staying at a gate. This reduces disk space consumption significantly as well as traffic needed to transfer the track to the web app.

The fetch API can serve an entire track, or only a tail of it starting from a given timestamp.

VATSIM API updates once in ~15s so native VATSIM tracks can look inaccurate, Tracked introduces a built-in spline interpolation adding computed points in between the real ones.
