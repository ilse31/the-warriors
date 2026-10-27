#include "ps2_runtime.h"
#include "raylib.h"
#include "runtime/ee_scheduler.h"
#include <iostream>
#include <exception>
#include <array>
#include <algorithm>

struct BootTrace {
    uint32_t pc;
    PS2Runtime::RecompiledFunction original = nullptr;
    unsigned hits = 0;
};

static std::array<BootTrace, 12> tracePoints{{{0x100008}, {0x10012c}, {0x10017c}, {0x100198}, {0x443530}, {0x43d320}, {0x1446d0}, {0x431378}, {0x431520}, {0x431538}, {0x228efc}, {0x1001c0}}};

static void traceBoot(uint8_t *memory, R5900Context *context, PS2Runtime *runtime) {
    auto point = std::find_if(tracePoints.begin(), tracePoints.end(),
                              [context](const BootTrace &p) { return p.pc == context->pc; });
    if (point == tracePoints.end() || !point->original) {
        throw std::runtime_error("Invalid boot trace dispatch");
    }
    const bool log = point->hits++ < 32;
    if (log) {
        std::cout << "[boot:enter] pc=0x" << std::hex << context->pc
                  << " ra=0x" << getRegU32(context, 31)
                  << " sp=0x" << getRegU32(context, 29)
                  << " a0=0x" << getRegU32(context, 4)
                  << " a1=0x" << getRegU32(context, 5)
                  << " a2=0x" << getRegU32(context, 6) << std::dec << std::endl;
    }
    point->original(memory, context, runtime);
    if (log) {
        std::cout << "[boot:return] from=0x" << std::hex << point->pc
                  << " pc=0x" << context->pc << " ra=0x" << getRegU32(context, 31)
                  << std::dec << std::endl;
    }
}

struct BootObserver {
    unsigned frames = 0;
};

int main() {
    try {
        BootObserver observer;
        PS2Runtime runtime;
        runtime.setMissingFunctionPolicy(PS2Runtime::MissingFunctionPolicy::Stop);
        runtime.setDebugUiCallbacks(
            [](PS2Runtime &, void *) {},
            [](PS2Runtime &game, void *state) {
                auto &observer = *static_cast<BootObserver *>(state);
                if (++observer.frames % 120 == 0) {
                    std::cout << "[boot] frame=" << observer.frames
                              << " pc=0x" << std::hex << game.m_debugPc.load()
                              << " ra=0x" << game.m_debugRa.load() << std::dec
                              << " dma=" << game.memory().dmaStartCount()
                              << " gif=" << game.memory().gifCopyCount()
                              << " gs=" << game.memory().gsWriteCount()
                              << " vif=" << game.memory().vifWriteCount() << std::endl;
                    for (const auto &thread : game.eeScheduler().snapshot().threads) {
                        std::cout << "[boot:thread] id=" << thread.id
                                  << " state=" << static_cast<int>(thread.status)
                                  << " wait=" << static_cast<int>(thread.waitReason)
                                  << " pc=0x" << std::hex << thread.pc
                                  << " ra=0x" << thread.ra << std::dec << std::endl;
                    }
                    if (observer.frames == 120) {
                        TakeScreenshot("boot-frame.png");
                    }
                }
            }, nullptr, &observer);
        if (!runtime.initialize("The Warriors — boot experiment")) {
            std::cerr << "[boot] Runtime initialization failed" << std::endl;
            return 1;
        }
        if (!runtime.loadELF("data/SLUS_212.15")) {
            std::cerr << "[boot] Cannot load data/SLUS_212.15" << std::endl;
            return 1;
        }
        if (runtime.cpu().pc != 0x100008u || !runtime.hasFunction(0x100008u)) {
            std::cerr << "[boot] ELF and generated function table do not match" << std::endl;
            return 1;
        }
        for (auto &point : tracePoints) {
            point.original = runtime.lookupFunction(point.pc);
            if (!point.original || !runtime.replaceFunction(point.pc, traceBoot)) {
                throw std::runtime_error("Boot trace target missing from generated table");
            }
        }
        runtime.run();
        return 0;
    } catch (const std::exception &error) {
        std::cerr << "[boot] Fatal: " << error.what() << std::endl;
        return 1;
    }
}
