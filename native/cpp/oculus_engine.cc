#include "oculus_engine.h"

#include <chrono>
#include <cstddef>
#include <cstdint>
#include <cstdlib>
#include <cstring>
#include <memory>
#include <string>
#include <vector>

#if defined(OCULUS_HAS_TESSERACT) || (__has_include(<tesseract/baseapi.h>) && (__has_include(<allheaders.h>) || __has_include(<leptonica/allheaders.h>)))
#ifndef OCULUS_HAS_TESSERACT
#define OCULUS_HAS_TESSERACT 1
#endif
#include <tesseract/baseapi.h>
#include <tesseract/renderer.h>
#if __has_include(<allheaders.h>)
#include <allheaders.h>
#elif __has_include(<leptonica/allheaders.h>)
#include <leptonica/allheaders.h>
#endif
#endif

#if defined(__APPLE__)
#include <CoreGraphics/CoreGraphics.h>
#endif

// Internal C++ RAII engine implementation
struct OculusEngineImpl {
    std::string tessdata_prefix;
#ifdef OCULUS_HAS_TESSERACT
    std::unique_ptr<tesseract::TessBaseAPI> tess;

    explicit OculusEngineImpl(const char* prefix)
        : tessdata_prefix(prefix ? prefix : ""),
          tess(std::make_unique<tesseract::TessBaseAPI>()) {}
#else
    explicit OculusEngineImpl(const char* prefix)
        : tessdata_prefix(prefix ? prefix : "") {}
#endif
};

// C++ internal structures to hold allocated memory safely
struct ResultHolder {
    std::string error_msg;
    std::vector<std::string> page_texts;
    std::vector<std::vector<std::string>> block_texts;
    std::vector<std::vector<OculusBlock>> page_blocks;
    std::vector<OculusPage> pages;
    OculusResult result{};
};

extern "C" {

OculusEngine* oculus_engine_create(const char* tessdata_prefix) {
    try {
        auto engine = std::make_unique<OculusEngineImpl>(tessdata_prefix);
        return reinterpret_cast<OculusEngine*>(engine.release());
    } catch (...) {
        return nullptr;
    }
}

void oculus_engine_destroy(OculusEngine* engine) {
    if (!engine) return;
    try {
        std::unique_ptr<OculusEngineImpl> ptr(reinterpret_cast<OculusEngineImpl*>(engine));
        // Automatically freed by RAII
    } catch (...) {
        // Prevent exceptions from escaping C ABI
    }
}

int32_t oculus_engine_preprocess(
    const uint8_t* in_data,
    size_t in_size,
    OculusPreprocessOptions options,
    uint8_t** out_data,
    size_t* out_size
) {
    if (!in_data || in_size == 0 || !out_data || !out_size) {
        return -1;
    }

    try {
#ifdef OCULUS_HAS_TESSERACT
        PIX* pix = pixReadMem(in_data, in_size);
        if (!pix) {
            return -2;
        }

        PIX* current = pix;

        // Stage 1: Grayscale conversion
        if (options.grayscale && pixGetDepth(current) != 8) {
            PIX* gray = pixConvertTo8(current, 0);
            if (gray) {
                if (current != pix) pixDestroy(&current);
                current = gray;
            }
        }

        // Stage 2: Orientation detection & rotation correction
        if (options.orientation_detect) {
            l_float32 upconf = 0.0f;
            l_float32 leftconf = 0.0f;
            if (pixOrientDetect(current, &upconf, &leftconf, 0, 0) == 0) {
                l_int32 orient = 0;
                if (makeOrientDecision(upconf, leftconf, 1.0f, 1.25f, &orient, 0) == 0) {
                    if (orient == 1) { // 90 deg clockwise
                        PIX* rot = pixRotate90(current, 1);
                        if (rot) {
                            if (current != pix) pixDestroy(&current);
                            current = rot;
                        }
                    } else if (orient == 2) { // 180 deg
                        PIX* rot = pixRotate180(nullptr, current);
                        if (rot) {
                            if (current != pix) pixDestroy(&current);
                            current = rot;
                        }
                    } else if (orient == 3) { // 270 deg (90 deg counter-clockwise)
                        PIX* rot = pixRotate90(current, -1);
                        if (rot) {
                            if (current != pix) pixDestroy(&current);
                            current = rot;
                        }
                    }
                }
            }
        }

        // Stage 3: Deskew correction
        if (options.deskew) {
            PIX* deskewed = pixDeskew(current, 2);
            if (deskewed) {
                if (current != pix) pixDestroy(&current);
                current = deskewed;
            }
        }

        // Stage 4: Denoise filter
        if (options.denoise && pixGetDepth(current) == 8) {
            PIX* denoised = pixBlockconv(current, 1, 1);
            if (denoised) {
                if (current != pix) pixDestroy(&current);
                current = denoised;
            }
        }

        // Stage 5: Contrast Normalization
        if (options.contrast_norm && pixGetDepth(current) == 8) {
            PIX* norm = pixContrastNorm(nullptr, current, 100, 100, 55, 2, 2);
            if (norm) {
                if (current != pix) pixDestroy(&current);
                current = norm;
            }
        }

        // Stage 6: Otsu Adaptive Thresholding / Binarization
        if (options.threshold) {
            PIX* binarized = nullptr;
            if (pixGetDepth(current) != 8) {
                PIX* gray = pixConvertTo8(current, 0);
                if (gray) {
                    pixOtsuAdaptiveThreshold(gray, 200, 200, 0, 0, 0.1f, nullptr, &binarized);
                    pixDestroy(&gray);
                }
            } else {
                pixOtsuAdaptiveThreshold(current, 200, 200, 0, 0, 0.1f, nullptr, &binarized);
            }

            if (binarized) {
                if (current != pix) pixDestroy(&current);
                current = binarized;
            }
        }

        // Write back to PNG in memory
        uint8_t* out_buf = nullptr;
        size_t out_len = 0;
        int write_err = pixWriteMemPng(&out_buf, &out_len, current, 0.0f);
        if (current != pix) pixDestroy(&current);
        pixDestroy(&pix);

        if (write_err != 0 || !out_buf) {
            return -3;
        }

        *out_data = out_buf;
        *out_size = out_len;
        return 0;
#else
        // Fallback: pass-through copy
        std::vector<uint8_t> buffer(in_data, in_data + in_size);
        auto* allocated = reinterpret_cast<uint8_t*>(malloc(buffer.size()));
        if (!allocated) return -1;
        std::memcpy(allocated, buffer.data(), buffer.size());
        *out_data = allocated;
        *out_size = buffer.size();
        return 0;
#endif
    } catch (...) {
        return -99;
    }
}

int32_t oculus_engine_ocr(
    OculusEngine* engine,
    const uint8_t* image_data,
    size_t image_size,
    const char* language,
    OculusPreprocessOptions options,
    OculusResult** out_result
) {
    if (!out_result) return -1;

    try {
        auto start_time = std::chrono::steady_clock::now();
        auto holder = std::make_unique<ResultHolder>();

        if (!image_data || image_size == 0) {
            holder->error_msg = "Empty or invalid image data buffer";
            holder->result.success = 0;
            holder->result.error_message = holder->error_msg.c_str();
            *out_result = &holder.release()->result;
            return 0;
        }

#ifdef OCULUS_HAS_TESSERACT
        OculusEngineImpl* impl = reinterpret_cast<OculusEngineImpl*>(engine);
        const char* datapath = (impl && !impl->tessdata_prefix.empty()) ? impl->tessdata_prefix.c_str() : nullptr;
        const char* lang = (language && strlen(language) > 0) ? language : "eng";

        // Read image from memory
        PIX* pix = pixReadMem(image_data, image_size);
        if (!pix) {
            holder->error_msg = "Failed to decode image data using Leptonica";
            holder->result.success = 0;
            holder->result.error_message = holder->error_msg.c_str();
            *out_result = &holder.release()->result;
            return 0;
        }

        PIX* processed_pix = pix;

        // Apply preprocessing pipeline
        if (options.grayscale && pixGetDepth(processed_pix) != 8) {
            PIX* gray = pixConvertTo8(processed_pix, 0);
            if (gray) {
                if (processed_pix != pix) pixDestroy(&processed_pix);
                processed_pix = gray;
            }
        }

        if (options.orientation_detect) {
            l_float32 upconf = 0.0f, leftconf = 0.0f;
            if (pixOrientDetect(processed_pix, &upconf, &leftconf, 0, 0) == 0) {
                l_int32 orient = 0;
                if (makeOrientDecision(upconf, leftconf, 1.0f, 1.25f, &orient, 0) == 0) {
                    if (orient == 1) {
                        PIX* rot = pixRotate90(processed_pix, 1);
                        if (rot) {
                            if (processed_pix != pix) pixDestroy(&processed_pix);
                            processed_pix = rot;
                        }
                    } else if (orient == 2) {
                        PIX* rot = pixRotate180(nullptr, processed_pix);
                        if (rot) {
                            if (processed_pix != pix) pixDestroy(&processed_pix);
                            processed_pix = rot;
                        }
                    } else if (orient == 3) {
                        PIX* rot = pixRotate90(processed_pix, -1);
                        if (rot) {
                            if (processed_pix != pix) pixDestroy(&processed_pix);
                            processed_pix = rot;
                        }
                    }
                }
            }
        }

        if (options.deskew) {
            PIX* deskewed = pixDeskew(processed_pix, 2);
            if (deskewed) {
                if (processed_pix != pix) pixDestroy(&processed_pix);
                processed_pix = deskewed;
            }
        }

        if (options.denoise && pixGetDepth(processed_pix) == 8) {
            PIX* denoised = pixBlockconv(processed_pix, 1, 1);
            if (denoised) {
                if (processed_pix != pix) pixDestroy(&processed_pix);
                processed_pix = denoised;
            }
        }

        if (options.contrast_norm && pixGetDepth(processed_pix) == 8) {
            PIX* norm = pixContrastNorm(nullptr, processed_pix, 100, 100, 55, 2, 2);
            if (norm) {
                if (processed_pix != pix) pixDestroy(&processed_pix);
                processed_pix = norm;
            }
        }

        if (options.threshold) {
            PIX* binarized = nullptr;
            PIX* gray = (pixGetDepth(processed_pix) == 8) ? processed_pix : pixConvertTo8(processed_pix, 0);
            if (gray) {
                pixOtsuAdaptiveThreshold(gray, 200, 200, 0, 0, 0.1f, nullptr, &binarized);
                if (gray != processed_pix) pixDestroy(&gray);
            }
            if (binarized) {
                if (processed_pix != pix) pixDestroy(&processed_pix);
                processed_pix = binarized;
            }
        }

        // Initialize Tesseract API
        tesseract::TessBaseAPI tess_api;
        if (tess_api.Init(datapath, lang) != 0) {
            if (std::strcmp(lang, "eng") != 0 && tess_api.Init(datapath, "eng") == 0) {
                // Succeeded with fallback
            } else {
                if (processed_pix != pix) pixDestroy(&processed_pix);
                pixDestroy(&pix);
                holder->error_msg = std::string("Failed to initialize Tesseract with language: ") + lang;
                holder->result.success = 0;
                holder->result.error_message = holder->error_msg.c_str();
                *out_result = &holder.release()->result;
                return 0;
            }
        }

        tess_api.SetImage(processed_pix);
        tess_api.Recognize(nullptr);

        // Extract UTF-8 text
        std::unique_ptr<char[]> text_ptr(tess_api.GetUTF8Text());
        std::string full_text = text_ptr ? std::string(text_ptr.get()) : "";
        float overall_confidence = static_cast<float>(tess_api.MeanTextConf());

        holder->page_texts.push_back(full_text);

        // Extract blocks / words with bounding boxes
        std::vector<OculusBlock> blocks;
        std::vector<std::string> block_str_store;

        std::unique_ptr<tesseract::ResultIterator> ri(tess_api.GetIterator());
        tesseract::PageIteratorLevel level = tesseract::RIL_WORD;
        if (ri) {
            do {
                std::unique_ptr<char[]> word(ri->GetUTF8Text(level));
                if (word) {
                    float conf = ri->Confidence(level);
                    int x1 = 0, y1 = 0, x2 = 0, y2 = 0;
                    ri->BoundingBox(level, &x1, &y1, &x2, &y2);

                    block_str_store.push_back(std::string(word.get()));

                    OculusBlock b{};
                    b.bbox.x = x1;
                    b.bbox.y = y1;
                    b.bbox.width = x2 - x1;
                    b.bbox.height = y2 - y1;
                    b.confidence = conf;
                    blocks.push_back(b);
                }
            } while (ri->Next(level));
        }

        if (processed_pix != pix) pixDestroy(&processed_pix);
        pixDestroy(&pix);

        holder->block_texts.push_back(std::move(block_str_store));
        holder->page_blocks.push_back(std::move(blocks));

        // Connect string pointers safely
        for (size_t i = 0; i < holder->page_blocks[0].size(); ++i) {
            holder->page_blocks[0][i].text = holder->block_texts[0][i].c_str();
        }

        OculusPage page{};
        page.page_number = 1;
        page.confidence = overall_confidence;
        page.full_text = holder->page_texts[0].c_str();
        page.block_count = holder->page_blocks[0].size();
        page.blocks = holder->page_blocks[0].data();

        holder->pages.push_back(page);

        auto end_time = std::chrono::steady_clock::now();
        uint64_t elapsed_ms = std::chrono::duration_cast<std::chrono::milliseconds>(end_time - start_time).count();

        holder->result.success = 1;
        holder->result.error_message = nullptr;
        holder->result.page_count = holder->pages.size();
        holder->result.pages = holder->pages.data();
        holder->result.processing_time_ms = elapsed_ms;

        *out_result = &holder.release()->result;
        return 0;
#else
        auto end_time = std::chrono::steady_clock::now();
        uint64_t elapsed_ms = std::chrono::duration_cast<std::chrono::milliseconds>(end_time - start_time).count();

        holder->error_msg = "Tesseract 5 headers not found during compilation.";
        holder->result.success = 0;
        holder->result.error_message = holder->error_msg.c_str();
        holder->result.processing_time_ms = elapsed_ms;

        *out_result = &holder.release()->result;
        return 0;
#endif
    } catch (const std::exception& e) {
        auto holder = std::make_unique<ResultHolder>();
        holder->error_msg = std::string("Unhandled C++ exception during OCR: ") + e.what();
        holder->result.success = 0;
        holder->result.error_message = holder->error_msg.c_str();
        *out_result = &holder.release()->result;
        return 0;
    } catch (...) {
        auto holder = std::make_unique<ResultHolder>();
        holder->error_msg = "Unknown C++ exception during OCR";
        holder->result.success = 0;
        holder->result.error_message = holder->error_msg.c_str();
        *out_result = &holder.release()->result;
        return 0;
    }
}

int32_t oculus_engine_pdf_page_count(const uint8_t* pdf_data, size_t pdf_size) {
    if (!pdf_data || pdf_size == 0) return -1;
    try {
#if defined(__APPLE__)
        CGDataProviderRef provider = CGDataProviderCreateWithData(nullptr, pdf_data, pdf_size, nullptr);
        if (!provider) return -1;
        CGPDFDocumentRef doc = CGPDFDocumentCreateWithProvider(provider);
        CGDataProviderRelease(provider);
        if (!doc) return -1;
        size_t count = CGPDFDocumentGetNumberOfPages(doc);
        CGPDFDocumentRelease(doc);
        return static_cast<int32_t>(count);
#else
        // No PDF engine on this platform yet (Windows/Linux): -9 = "not supported", not "bad PDF".
        return -9;
#endif
    } catch (...) {
        return -1;
    }
}

int32_t oculus_engine_render_pdf_page(
    const uint8_t* pdf_data,
    size_t pdf_size,
    int32_t page_number,
    int32_t dpi,
    uint8_t** out_png_data,
    size_t* out_png_size
) {
    if (!pdf_data || pdf_size == 0 || !out_png_data || !out_png_size || page_number < 1) {
        return -1;
    }

    try {
#if defined(__APPLE__)
        CGDataProviderRef provider = CGDataProviderCreateWithData(nullptr, pdf_data, pdf_size, nullptr);
        if (!provider) return -2;
        CGPDFDocumentRef doc = CGPDFDocumentCreateWithProvider(provider);
        CGDataProviderRelease(provider);
        if (!doc) return -2;

        CGPDFPageRef page = CGPDFDocumentGetPage(doc, static_cast<size_t>(page_number));
        if (!page) {
            CGPDFDocumentRelease(doc);
            return -3;
        }

        CGRect media_box = CGPDFPageGetBoxRect(page, kCGPDFMediaBox);
        CGPDFBox active_box = kCGPDFMediaBox;
        if (CGRectIsEmpty(media_box) || media_box.size.width <= 0.0 || media_box.size.height <= 0.0) {
            media_box = CGPDFPageGetBoxRect(page, kCGPDFCropBox);
            active_box = kCGPDFCropBox;
        }
        if (CGRectIsEmpty(media_box) || media_box.size.width <= 0.0 || media_box.size.height <= 0.0) {
            media_box = CGPDFPageGetBoxRect(page, kCGPDFBleedBox);
            active_box = kCGPDFBleedBox;
        }
        if (CGRectIsEmpty(media_box) || media_box.size.width <= 0.0 || media_box.size.height <= 0.0) {
            media_box = CGRectMake(0, 0, 612.0, 792.0);
        }

        double target_dpi = dpi > 0 ? static_cast<double>(dpi) : 300.0;
        double scale = target_dpi / 72.0;
        size_t width = static_cast<size_t>(media_box.size.width * scale);
        size_t height = static_cast<size_t>(media_box.size.height * scale);

        if (width == 0 || height == 0) {
            width = static_cast<size_t>(612.0 * scale);
            height = static_cast<size_t>(792.0 * scale);
        }

        CGColorSpaceRef color_space = CGColorSpaceCreateDeviceRGB();
        size_t bytes_per_row = width * 4;
        std::vector<uint8_t> bitmap_bytes(height * bytes_per_row, 255);

        CGContextRef ctx = CGBitmapContextCreate(
            bitmap_bytes.data(),
            width,
            height,
            8,
            bytes_per_row,
            color_space,
            kCGImageAlphaPremultipliedLast
        );
        CGColorSpaceRelease(color_space);

        if (!ctx) {
            CGPDFDocumentRelease(doc);
            return -5;
        }

        // Fill white background
        CGContextSetRGBFillColor(ctx, 1.0, 1.0, 1.0, 1.0);
        CGContextFillRect(ctx, CGRectMake(0, 0, static_cast<CGFloat>(width), static_cast<CGFloat>(height)));

        // Scale and draw PDF page using Apple drawing transform (preserves rotation & origin offset)
        CGContextSaveGState(ctx);
        CGAffineTransform transform = CGPDFPageGetDrawingTransform(
            page,
            active_box,
            CGRectMake(0, 0, static_cast<CGFloat>(width), static_cast<CGFloat>(height)),
            0,
            true
        );
        CGContextConcatCTM(ctx, transform);
        CGContextDrawPDFPage(ctx, page);
        CGContextRestoreGState(ctx);

        CGContextRelease(ctx);
        CGPDFDocumentRelease(doc);

#ifdef OCULUS_HAS_TESSERACT
        // Convert to Leptonica PIX
        PIX* pix = pixCreate(static_cast<l_int32>(width), static_cast<l_int32>(height), 32);
        if (!pix) return -6;

        uint32_t* pix_data = pixGetData(pix);
        int wpl = pixGetWpl(pix);
        for (size_t y = 0; y < height; ++y) {
            const uint8_t* src_row = bitmap_bytes.data() + y * bytes_per_row;
            uint32_t* dst_row = pix_data + y * wpl;
            for (size_t x = 0; x < width; ++x) {
                uint8_t r = src_row[x * 4 + 0];
                uint8_t g = src_row[x * 4 + 1];
                uint8_t b = src_row[x * 4 + 2];
                uint8_t a = src_row[x * 4 + 3];
                dst_row[x] = (static_cast<uint32_t>(r) << 24) |
                             (static_cast<uint32_t>(g) << 16) |
                             (static_cast<uint32_t>(b) << 8) |
                             static_cast<uint32_t>(a);
            }
        }

        uint8_t* out_buf = nullptr;
        size_t out_len = 0;
        int write_res = pixWriteMemPng(&out_buf, &out_len, pix, 0.0f);
        pixDestroy(&pix);

        if (write_res != 0 || !out_buf) {
            return -7;
        }

        *out_png_data = out_buf;
        *out_png_size = out_len;
        return 0;
#else
        return -8;
#endif
#else
        return -9;
#endif
    } catch (...) {
        return -99;
    }
}

int32_t oculus_engine_create_searchable_pdf(
    OculusEngine* engine,
    const uint8_t* image_data,
    size_t image_size,
    const char* language,
    const char* output_pdf_base
) {
    if (!engine || !image_data || image_size == 0 || !output_pdf_base) return -1;
    try {
#ifdef OCULUS_HAS_TESSERACT
        OculusEngineImpl* impl = reinterpret_cast<OculusEngineImpl*>(engine);
        const char* datapath = (!impl->tessdata_prefix.empty()) ? impl->tessdata_prefix.c_str() : nullptr;
        const char* lang = (language && strlen(language) > 0) ? language : "eng";

        tesseract::TessBaseAPI tess_api;
        if (tess_api.Init(datapath, lang) != 0) {
            if (std::strcmp(lang, "eng") != 0 && tess_api.Init(datapath, "eng") == 0) {
                // English fallback
            } else {
                return -2;
            }
        }

        PIX* pix = pixReadMem(image_data, image_size);
        if (!pix) return -3;

        auto renderer = std::make_unique<tesseract::TessPDFRenderer>(output_pdf_base, datapath, false);
        if (!renderer->BeginDocument("OCULUS Searchable PDF")) {
            pixDestroy(&pix);
            return -4;
        }

        tess_api.SetImage(pix);
        tess_api.Recognize(nullptr);
        bool added = renderer->AddImage(&tess_api);
        renderer->EndDocument();
        pixDestroy(&pix);

        return added ? 0 : -5;
#else
        return -6;
#endif
    } catch (...) {
        return -99;
    }
}

void oculus_free_result(OculusResult* result) {
    if (!result) return;
    try {
        ResultHolder* holder = reinterpret_cast<ResultHolder*>(
            reinterpret_cast<char*>(result) - offsetof(ResultHolder, result)
        );
        std::unique_ptr<ResultHolder> ptr(holder);
        // Destructor reclaims all vectors and strings safely via RAII
    } catch (...) {
        // Suppress
    }
}

void oculus_free_buffer(uint8_t* buffer) {
    if (buffer) {
        try {
#ifdef OCULUS_HAS_TESSERACT
            lept_free(buffer);
#else
            free(buffer);
#endif
        } catch (...) {
            // Suppress
        }
    }
}

} // extern "C"
