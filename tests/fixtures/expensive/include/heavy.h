
#pragma once
#include <iostream>
#include <vector>
#include <string>
#include <map>

template <typename T>
struct HeavyTemplate {
    T value;
    void do_something() {
        std::vector<T> vec;
        for(int i=0; i<100; ++i) vec.push_back(value);
    }
};

inline void heavy_func() {
    HeavyTemplate<int> h;
    h.do_something();
}
