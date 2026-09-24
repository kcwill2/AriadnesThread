#pragma once

#include "thing_types.h"
#include "heavy.h" // Intentional collateral dependency!

class Thing {
public:
    explicit Thing(ThingConfig config);
    int get_id() const;

private:
    ThingConfig config_;
};
