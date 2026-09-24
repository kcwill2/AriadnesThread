import os

base_path = r"c:\Users\Kurt\Code\AriadnesThread\tests\fixtures\expensive"
os.makedirs(os.path.join(base_path, "include"), exist_ok=True)
os.makedirs(os.path.join(base_path, "src"), exist_ok=True)

heavy_h = """
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
"""

thing_h = """
#pragma once
#include "heavy.h"

class Thing {
public:
    void use_heavy() {
        heavy_func();
    }
};
"""

consumer1_cpp = """
#include "../include/thing.h"

int main() {
    Thing t;
    t.use_heavy();
    return 0;
}
"""

consumer2_cpp = """
#include "../include/thing.h"
#include "../include/heavy.h"

void another_func() {
    Thing t;
    t.use_heavy();
    heavy_func();
}
"""

with open(os.path.join(base_path, "include", "heavy.h"), "w") as f:
    f.write(heavy_h)

with open(os.path.join(base_path, "include", "thing.h"), "w") as f:
    f.write(thing_h)

with open(os.path.join(base_path, "src", "consumer1.cpp"), "w") as f:
    f.write(consumer1_cpp)

with open(os.path.join(base_path, "src", "consumer2.cpp"), "w") as f:
    f.write(consumer2_cpp)

print("Created expensive fixture")
