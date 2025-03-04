//! `flash.media.Sound` builtin/prototype

use crate::avm2::activation::Activation;
use crate::avm2::class::{Class, ClassAttributes};
use crate::avm2::method::{Method, NativeMethodImpl};
use crate::avm2::object::{sound_allocator, Object, SoundChannelObject, TObject};
use crate::avm2::value::Value;
use crate::avm2::Error;
use crate::avm2::Multiname;
use crate::avm2::Namespace;
use crate::avm2::QName;
use crate::backend::navigator::Request;
use crate::character::Character;
use crate::display_object::SoundTransform;
use crate::{avm2_stub_getter, avm2_stub_method};
use gc_arena::{GcCell, MutationContext};
use swf::{SoundEvent, SoundInfo};

/// Implements `flash.media.Sound`'s instance constructor.
pub fn instance_init<'gc>(
    activation: &mut Activation<'_, 'gc>,
    this: Option<Object<'gc>>,
    _args: &[Value<'gc>],
) -> Result<Value<'gc>, Error<'gc>> {
    if let Some(this) = this {
        activation.super_init(this, &[])?;

        if this.as_sound().is_none() {
            let class_object = this
                .instance_of()
                .ok_or("Attempted to construct Sound on a bare object.")?;

            if let Some((movie, symbol)) = activation
                .context
                .library
                .avm2_class_registry()
                .class_symbol(class_object)
            {
                if let Some(Character::Sound(sound)) = activation
                    .context
                    .library
                    .library_for_movie_mut(movie)
                    .character_by_id(symbol)
                {
                    this.set_sound(activation.context.gc_context, *sound);
                } else {
                    tracing::warn!("Attempted to construct subclass of Sound, {}, which is associated with non-Sound character {}", class_object.inner_class_definition().read().name().local_name(), symbol);
                }
            }
        }
    }

    Ok(Value::Undefined)
}

/// Implements `flash.media.Sound`'s class constructor.
pub fn class_init<'gc>(
    _activation: &mut Activation<'_, 'gc>,
    _this: Option<Object<'gc>>,
    _args: &[Value<'gc>],
) -> Result<Value<'gc>, Error<'gc>> {
    Ok(Value::Undefined)
}

/// Implements `Sound.bytesTotal`
pub fn bytes_total<'gc>(
    activation: &mut Activation<'_, 'gc>,
    this: Option<Object<'gc>>,
    _args: &[Value<'gc>],
) -> Result<Value<'gc>, Error<'gc>> {
    if let Some(sound) = this.and_then(|this| this.as_sound()) {
        if let Some(length) = activation.context.audio.get_sound_size(sound) {
            return Ok((length).into());
        }
    }

    Ok(Value::Undefined)
}

/// Implements `Sound.isBuffering`
pub fn is_buffering<'gc>(
    activation: &mut Activation<'_, 'gc>,
    _this: Option<Object<'gc>>,
    _args: &[Value<'gc>],
) -> Result<Value<'gc>, Error<'gc>> {
    avm2_stub_getter!(activation, "flash.media.Sound", "isBuffering");
    //STUB: We do not yet support network-loaded sounds.
    Ok(false.into())
}

/// Implements `Sound.url`
pub fn url<'gc>(
    activation: &mut Activation<'_, 'gc>,
    _this: Option<Object<'gc>>,
    _args: &[Value<'gc>],
) -> Result<Value<'gc>, Error<'gc>> {
    avm2_stub_getter!(activation, "flash.media.Sound", "url");
    //STUB: We do not yet support network-loaded sounds.
    Ok(Value::Null)
}

/// Implements `Sound.length`
pub fn length<'gc>(
    activation: &mut Activation<'_, 'gc>,
    this: Option<Object<'gc>>,
    _args: &[Value<'gc>],
) -> Result<Value<'gc>, Error<'gc>> {
    if let Some(sound) = this.and_then(|this| this.as_sound()) {
        if let Some(duration) = activation.context.audio.get_sound_duration(sound) {
            return Ok((duration).into());
        }
    }

    Ok(Value::Undefined)
}

/// Implements `Sound.play`
pub fn play<'gc>(
    activation: &mut Activation<'_, 'gc>,
    this: Option<Object<'gc>>,
    args: &[Value<'gc>],
) -> Result<Value<'gc>, Error<'gc>> {
    if let Some(sound) = this.and_then(|this| this.as_sound()) {
        let position = args
            .get(0)
            .cloned()
            .unwrap_or_else(|| 0.0.into())
            .coerce_to_number(activation)?;
        let num_loops = args
            .get(1)
            .cloned()
            .unwrap_or_else(|| 0.into())
            .coerce_to_i32(activation)?;
        let sound_transform = args.get(2).cloned().unwrap_or(Value::Null).as_object();

        if let Some(duration) = activation.context.audio.get_sound_duration(sound) {
            if position > duration {
                return Ok(Value::Null);
            }
        }

        let in_sample = if position > 0.0 {
            Some((position / 1000.0 * 44100.0) as u32)
        } else {
            None
        };

        let sound_info = SoundInfo {
            event: SoundEvent::Start,
            in_sample,
            out_sample: None,
            num_loops: num_loops.max(1) as u16,
            envelope: None,
        };

        if let Some(instance) = activation
            .context
            .start_sound(sound, &sound_info, None, None)
        {
            if let Some(sound_transform) = sound_transform {
                let st = SoundTransform::from_avm2_object(activation, sound_transform)?;
                activation.context.set_local_sound_transform(instance, st);
            }

            let sound_channel = SoundChannelObject::from_sound_instance(activation, instance)?;

            activation
                .context
                .attach_avm2_sound_channel(instance, sound_channel);

            return Ok(sound_channel.into());
        }
    }

    Ok(Value::Null)
}

/// `Sound.extract`
pub fn extract<'gc>(
    activation: &mut Activation<'_, 'gc>,
    _this: Option<Object<'gc>>,
    _args: &[Value<'gc>],
) -> Result<Value<'gc>, Error<'gc>> {
    avm2_stub_method!(activation, "flash.media.Sound", "extract");
    Ok(Value::Undefined)
}

/// `Sound.close`
pub fn close<'gc>(
    activation: &mut Activation<'_, 'gc>,
    _this: Option<Object<'gc>>,
    _args: &[Value<'gc>],
) -> Result<Value<'gc>, Error<'gc>> {
    avm2_stub_method!(activation, "flash.media.Sound", "close");
    Ok(Value::Undefined)
}

/// `Sound.load`
pub fn load<'gc>(
    activation: &mut Activation<'_, 'gc>,
    this: Option<Object<'gc>>,
    args: &[Value<'gc>],
) -> Result<Value<'gc>, Error<'gc>> {
    if let Some(this) = this {
        let url_request = match args.get(0) {
            Some(Value::Object(request)) => request,
            // This should never actually happen
            _ => return Ok(Value::Undefined),
        };

        let url = url_request
            .get_property(&Multiname::public("url"), activation)?
            .coerce_to_string(activation)?;

        // TODO: context parameter currently unused.
        let _sound_context = args.get(1);
        if _sound_context.is_some() {
            avm2_stub_method!(activation, "flash.media.Sound", "load", "with context");
        }

        let future = activation.context.load_manager.load_sound_avm2(
            activation.context.player.clone(),
            this,
            // FIXME: Set options from the `URLRequest`.
            Request::get(url.to_string()),
        );
        activation.context.navigator.spawn_future(future);
    }
    Ok(Value::Undefined)
}

/// `Sound.loadCompressedDataFromByteArray`
pub fn load_compressed_data_from_byte_array<'gc>(
    activation: &mut Activation<'_, 'gc>,
    _this: Option<Object<'gc>>,
    _args: &[Value<'gc>],
) -> Result<Value<'gc>, Error<'gc>> {
    avm2_stub_method!(
        activation,
        "flash.media.Sound",
        "loadCompressedDataFromByteArray"
    );
    Ok(Value::Undefined)
}

/// `Sound.loadPCMFromByteArray`
pub fn load_pcm_from_byte_array<'gc>(
    activation: &mut Activation<'_, 'gc>,
    _this: Option<Object<'gc>>,
    _args: &[Value<'gc>],
) -> Result<Value<'gc>, Error<'gc>> {
    avm2_stub_method!(activation, "flash.media.Sound", "loadPCMFromByteArray");
    Ok(Value::Undefined)
}

/// Construct `Sound`'s class.
pub fn create_class<'gc>(mc: MutationContext<'gc, '_>) -> GcCell<'gc, Class<'gc>> {
    let class = Class::new(
        QName::new(Namespace::package("flash.media"), "Sound"),
        Some(Multiname::new(
            Namespace::package("flash.events"),
            "EventDispatcher",
        )),
        Method::from_builtin(instance_init, "<Sound instance initializer>", mc),
        Method::from_builtin(class_init, "<Sound class initializer>", mc),
        mc,
    );

    let mut write = class.write(mc);

    write.set_attributes(ClassAttributes::SEALED);
    write.set_instance_allocator(sound_allocator);

    const PUBLIC_INSTANCE_PROPERTIES: &[(
        &str,
        Option<NativeMethodImpl>,
        Option<NativeMethodImpl>,
    )] = &[
        ("bytesLoaded", Some(bytes_total), None),
        ("bytesTotal", Some(bytes_total), None),
        ("isBuffering", Some(is_buffering), None),
        ("isURLInaccessible", Some(is_buffering), None),
        ("url", Some(url), None),
        ("length", Some(length), None),
    ];
    write.define_public_builtin_instance_properties(mc, PUBLIC_INSTANCE_PROPERTIES);

    const PUBLIC_INSTANCE_METHODS: &[(&str, NativeMethodImpl)] = &[
        ("play", play),
        ("extract", extract),
        ("load", load),
        ("close", close),
        (
            "loadCompressedDataFromByteArray",
            load_compressed_data_from_byte_array,
        ),
        ("loadPCMFromByteArray", load_pcm_from_byte_array),
    ];
    write.define_public_builtin_instance_methods(mc, PUBLIC_INSTANCE_METHODS);

    class
}
