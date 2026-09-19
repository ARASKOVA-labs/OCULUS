#ifndef OCULUS_ENGINE_H
#define OCULUS_ENGINE_H

#include <stddef.h>
#include <stdint.h>

#ifdef __cplusplus
extern "C" {
#endif

typedef struct OculusEngine OculusEngine;

typedef struct OculusPreprocessOptions {
    int32_t grayscale;
    int32_t denoise;
    int32_t deskew;
    int32_t threshold;
    int32_t orientation_detect;
    int32_t contrast_norm;
} OculusPreprocessOptions;

typedef struct OculusBoundingBox {
    int32_t x;
    int32_t y;
    int32_t width;
    int32_t height;
} OculusBoundingBox;

typedef struct OculusBlock {
    OculusBoundingBox bbox;
    float confidence;
    const char* text;
} OculusBlock;

typedef struct OculusPage {
    int32_t page_number;
    float confidence;
    const char* full_text;
    size_t block_count;
    OculusBlock* blocks;
} OculusPage;

typedef struct OculusResult {
    int32_t success;
    const char* error_message;
    size_t page_count;
    OculusPage* pages;
    uint64_t processing_time_ms;
} OculusResult;

/* Engine lifecycle */
OculusEngine* oculus_engine_create(const char* tessdata_prefix);
void oculus_engine_destroy(OculusEngine* engine);

/* Image Preprocessing */
int32_t oculus_engine_preprocess(
    const uint8_t* in_data,
    size_t in_size,
    OculusPreprocessOptions options,
    uint8_t** out_data,
    size_t* out_size
);

/* Run OCR */
int32_t oculus_engine_ocr(
    OculusEngine* engine,
    const uint8_t* image_data,
    size_t image_size,
    const char* language,
    OculusPreprocessOptions options,
    OculusResult** out_result
);

/* PDF Operations */
int32_t oculus_engine_pdf_page_count(const uint8_t* pdf_data, size_t pdf_size);
int32_t oculus_engine_render_pdf_page(
    const uint8_t* pdf_data,
    size_t pdf_size,
    int32_t page_number,
    int32_t dpi,
    uint8_t** out_png_data,
    size_t* out_png_size
);
int32_t oculus_engine_create_searchable_pdf(
    OculusEngine* engine,
    const uint8_t* image_data,
    size_t image_size,
    const char* language,
    const char* output_pdf_base
);

/* Memory cleanup */
void oculus_free_result(OculusResult* result);
void oculus_free_buffer(uint8_t* buffer);

#ifdef __cplusplus
}
#endif

#endif /* OCULUS_ENGINE_H */
