use glutin::config::{Config, ConfigTemplateBuilder};
use glutin::context::{ContextApi, ContextAttributesBuilder, GlProfile, Version};
use glutin::prelude::*;
use winit::window::WindowBuilder;

pub fn window_builder() -> WindowBuilder {
    WindowBuilder::new().with_transparent(true)
}

pub fn config_template_builder() -> ConfigTemplateBuilder {
    // The template will match only the configurations supporting rendering
    // to windows.
    //
    // XXX We force transparency only on macOS, given that EGL on X11 doesn't
    // have it, but we still want to show window. The macOS situation is like
    // that, because we can query only one config at a time on it, but all
    // normal platforms will return multiple configs, so we can find the config
    // with transparency ourselves inside the `reduce`.
    ConfigTemplateBuilder::new()
        .with_alpha_size(8)
        .with_transparency(cfg!(cgl_backend))
}

pub fn config_picker(configs: Box<dyn Iterator<Item = Config> + '_>) -> Config {
    // Find the config with the maximum number of samples, so our triangle will
    // be smooth.
    configs
        .reduce(|accum, config| {
            let transparency_check =
                config.supports_transparency().unwrap_or(false) & !accum.supports_transparency().unwrap_or(false);

            if transparency_check || config.num_samples() > accum.num_samples() {
                config
            } else {
                accum
            }
        })
        .unwrap()
}

pub fn context_attributes_builder() -> ContextAttributesBuilder {
    ContextAttributesBuilder::new()
        .with_profile(GlProfile::Core)
        .with_context_api(ContextApi::OpenGl(Some(Version::new(4, 5))))
}
