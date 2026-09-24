#pragma once

// Unnecessary collateral dependency defining heavy structures
struct HeavyData {
    double payload[10000];
    char buffer[65536];
};

inline void heavy_computation(HeavyData* data) {
    (void)data;
}
