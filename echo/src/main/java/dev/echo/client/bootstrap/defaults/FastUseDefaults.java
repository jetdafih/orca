package dev.echo.client.bootstrap.defaults;

import dev.echo.client.bootstrap.CatalogText;
import dev.echo.client.features.impl.utility.FastUseFeature;

/** Constructs, assigns, collects and post-initializes Fast Use. */
public final class FastUseDefaults {
    private FastUseDefaults() {
    }

    /** Runs only after the bootstrap has allocated every feature. */
    public static void initialize(FastUseFeature feature) {
        dev.echo.client.setting.FloatRangeSetting preparedDelay = new dev.echo.client.setting.FloatRangeSetting(CatalogText.literal("Delay"), 0.0f, 0.0f, 0.0f, 4.0f, 1.0f, CatalogText.literal(" ticks"));

        FastUseFeature.delay = preparedDelay;
        FastUseFeature.exceptionDelay = new dev.echo.client.setting.FloatRangeSetting(CatalogText.literal("Exception Delay"), 0.0f, 0.0f, 0.0f, 4.0f, 1.0f, CatalogText.literal(" ticks"));

        feature.settings.clear();
        dev.echo.client.bootstrap.CatalogSettings.append(feature, dev.echo.client.features.impl.utility.FastUseFeature.delay);
        dev.echo.client.bootstrap.CatalogSettings.append(feature, dev.echo.client.features.impl.utility.FastUseFeature.items);
        dev.echo.client.bootstrap.CatalogSettings.append(feature, dev.echo.client.features.impl.utility.FastUseFeature.exceptions);
        dev.echo.client.bootstrap.CatalogSettings.append(feature, dev.echo.client.features.impl.utility.FastUseFeature.exceptionDelay);
        feature.postInitialize();
    }
}
