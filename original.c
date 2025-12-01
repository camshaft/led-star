#include "FastLED.h"
#include <stdlib.h>

#define DATA_PIN 3
#define CLOCK_PIN 13

// How many leds in your strip?
#define NUM_LEDS 384
#define NUM_SPINES 12
#define SPINE_LED_LENGTH 32
#define TIME_DELAY 25
#define MAX 255

#define ROTATE_LENGTH 8

#define STREAK_LENGTH 10
#define STREAK_CYCLE 20
#define STREAK_OFFSET 7
#define STREAK_VARIANCE 2

#define TRIANGLE_LENGTH 2

#define HUE_FUNCTION fn_rotate
#define SATURATION_FUNCTION fn_triangle
#define LIGHTNESS_FUNCTION fn_streak

// Define the array of leds
CRGB leds[NUM_LEDS];

uint8_t fn_rotate(uint16_t t, uint8_t spine, uint8_t idx)
{
    return (uint8_t)((t / ROTATE_LENGTH + spine) / (float)NUM_SPINES * MAX) % MAX;
}

uint8_t fn_streak(uint16_t t, uint8_t spine, uint8_t idx)
{
    uint8_t offset = idx - t - 1 + spine * STREAK_OFFSET;
    return (uint8_t)(max(0, offset % STREAK_CYCLE + 1 - STREAK_LENGTH) / (float)STREAK_LENGTH * MAX);
}

uint8_t fn_triangle(uint16_t t, uint8_t spine, uint8_t idx)
{
    int value = t / TRIANGLE_LENGTH % MAX * 2 - MAX;
    return abs(value);
}

void loop()
{
    static uint16_t t = 0;

    uint8_t spine;
    uint8_t idx;
    for (uint8_t i = 0; i < NUM_LEDS / 2; i++)
    {
        spine = i / (SPINE_LED_LENGTH / 2);
        idx = i % (SPINE_LED_LENGTH / 2);
        leds[spine * SPINE_LED_LENGTH / 2 + i] =
            leds[(spine + 1) * SPINE_LED_LENGTH - 1 - idx] =
                CHSV(
                    HUE_FUNCTION(t, spine, idx),
                    SATURATION_FUNCTION(t, spine, idx),
                    LIGHTNESS_FUNCTION(t, spine, idx));
    }
    FastLED.show(128);

    t++;
    delay(TIME_DELAY);
}

void setup()
{
    Serial.begin(57600);
    LEDS.addLeds<WS2812, DATA_PIN, GRB>(leds, NUM_LEDS);
    LEDS.setBrightness(84);
}
