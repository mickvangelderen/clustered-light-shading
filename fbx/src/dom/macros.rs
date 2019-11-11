macro_rules! impl_properties70 {
    (@read [f64; 3] $node: expr) => {
        [
            $node.properties[4].to_f64_exact(),
            $node.properties[5].to_f64_exact(),
            $node.properties[6].to_f64_exact(),
        ]
    };

    (@read f64 $node: expr) => {
        $node.properties[4].to_f64_exact()
    };

    (@read i32 $node: expr) => {
        $node.properties[4].to_i32()
    };

    (@read u8 $node: expr) => {
        $node.properties[4].to_u8()
    };

    (@read QuaternionInterpolationMode $node: expr) => {
        $crate::types::QuaternionInterpolationMode::from_i32($node.properties[4].to_i32()).unwrap()
    };

    (@read RotationOrder $node: expr) => {
        $crate::types::RotationOrder::from_i32($node.properties[4].to_i32()).unwrap()
    };

    (@read String $node: expr) => {
        $node.properties[4].as_str().to_string()
    };

    ($Properties: ident {
        $(
            $($name: tt)|+ => $field: ident: $ty: tt = $default: expr,
        )*
    }) => {
        #[derive(Debug)]
        pub struct $Properties {
            $(
                pub $field: $ty,
            )*
        }

        impl Default for $Properties {
            fn default() -> Self {
                Self {
                    $(
                        $field: $default,
                    )*
                }
            }
        }

        impl $Properties {
            pub fn from_fbx(node: &$crate::tree::Node, stack: &mut Vec<String>) -> Self {
                let mut properties = Self::default();

                for node in node.children.iter() {
                    stack.push(node.name.clone());

                    assert_eq!(node.name.as_str(), "P");

                    match node.properties[0].as_str() {
                        $(
                            $($name)|+ => properties.$field = impl_properties70!(@read $ty node),
                        )*
                        unknown => {
                            eprintln!(concat!("Unknown ", stringify!($Properties), " property {:?}"), unknown);
                        }
                    }

                    stack.pop();
                }

                properties
            }
        }
    };
}
