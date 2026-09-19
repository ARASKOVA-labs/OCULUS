#include "oculus_engine.h"
#include <cassert>
#include <iostream>

int main() {
    std::cout << "[OCULUS Native Test] Initializing engine..." << std::endl;
    OculusEngine* engine = oculus_engine_create(nullptr);
    assert(engine != nullptr);

    // Test with invalid buffer to check error propagation
    OculusPreprocessOptions opts{1, 0, 0, 1};
    OculusResult* result = nullptr;
    int32_t rc = oculus_engine_ocr(engine, nullptr, 0, "eng", opts, &result);
    assert(rc == 0);
    assert(result != nullptr);
    assert(result->success == 0);
    assert(result->error_message != nullptr);

    std::cout << "[OCULUS Native Test] Error propagation verified: " << result->error_message << std::endl;
    oculus_free_result(result);
    oculus_engine_destroy(engine);

    std::cout << "[OCULUS Native Test] Passed!" << std::endl;
    return 0;
}
