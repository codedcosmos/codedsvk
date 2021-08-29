use xmltree::Element;
use std::fs::File;
use std::io::{BufReader, Write};
use std::collections::HashMap;

struct APIConstant {
    variable_type: String,
    value: String,
    name: String,
}

struct VulkanEnum {
    name: String,
    values: HashMap<String, i32>,
}

fn main() {
    println!("Building based on vkxml");
    let file = File::open("xmlsrc/vk.xml").expect("Failed to find xmlsrc/vk.xml");
    let file = BufReader::new(file);


    let mut element = Element::parse(file).unwrap();


    // Extracted vulkan contents
    let mut api_constants = HashMap::new();
    let mut enums = Vec::new();
    let mut handles = Vec::new();


    for child in &element.children {
        if let Some(child) = child.as_element() {
            if child.name.eq("enums") {

                // Redeclare name
                let vkenum = child;

                if let Some(name) = vkenum.attributes.get("name") {
                    match name.as_str() {
                        "API Constants" => {
                            println!("Constants found! {:?}", vkenum);

                            for elem in &vkenum.children {
                                if let Some(elem) = elem.as_element() {
                                    if let (Some(variable_type), Some(value), Some(name)) = (elem.attributes.get("type"), elem.attributes.get("value"), elem.attributes.get("name")) {
                                        api_constants.insert(name.clone(), APIConstant {
                                            variable_type: variable_type.to_owned(),
                                            value: value.to_owned(),
                                            name: name.to_owned()
                                        });
                                        println!("Got constant {} {} {}", variable_type, name, value);
                                    } else if let (Some(name), Some(alias)) = (elem.attributes.get("name"), elem.attributes.get("alias")) {
                                        if let Some(api_constant) = api_constants.get(alias) {
                                            api_constants.insert(name.clone(), APIConstant {
                                                variable_type: api_constant.variable_type.clone(),
                                                value: api_constant.value.clone(),
                                                name: name.to_owned()
                                            });
                                            println!("Got alias {} {}", name, alias);
                                        } else {
                                            panic!("Failed to find matching alias!");
                                        }
                                    } else {
                                        panic!("Unsure what to do with alias constant!");
                                    }
                                }
                            }
                        }

                        _ => {
                            println!("Got enum! {}", name);

                            let mut newenum = VulkanEnum {
                                name: name.clone(),
                                values: HashMap::new()
                            };

                            for enum_states in &vkenum.children {
                                if let Some(elem) = enum_states.as_element() {

                                    if let (Some(value), Some(name)) = (elem.attributes.get("value"), elem.attributes.get("name")) {
                                        if let Ok(value) = value.parse::<i32>() {
                                            newenum.values.insert(name.clone(), value);
                                        }
                                    }

                                }
                            }

                            enums.push(newenum);
                        }
                    }

                }

            } else if child.name.eq("types") {
                for vktype in &child.children {
                    if let Some(element) = vktype.as_element() {
                        if let Some(category) = element.attributes.get("category") {
                            match category.as_str() {
                                "define" => {
                                    //println!("Definition {:?}", element);
                                }
                                "basetype" => {
                                    //println!("Basetype {:?}", element);
                                }
                                "bitmask" => {
                                    //println!("Bitmask {:?}", element);
                                }
                                "handle" => {
                                    //println!("Handle {:?}", element);
                                    let mut handle_name = None;
                                    let mut handle_type = None;

                                    for child in &element.children {
                                        if let Some(child_handle_name_definition) = child.as_element() {
                                            if child_handle_name_definition.name.eq("type") {
                                                if let Some(handle_text) = child_handle_name_definition.children.first() {
                                                    if let Some(handle_text) = handle_text.as_text() {
                                                        handle_type = Some(handle_text);
                                                    }
                                                }
                                            } else if child_handle_name_definition.name.eq("name") {
                                                if let Some(handle_text) = child_handle_name_definition.children.first() {
                                                    if let Some(handle_text) = handle_text.as_text() {
                                                        handle_name = Some(handle_text);
                                                    }
                                                }
                                            }
                                        }
                                    }

                                    println!("{} {}", handle_name.unwrap_or("None"), handle_type.unwrap_or("None"));

                                    if let (Some(handle_type), Some(handle_name)) = (handle_type, handle_name) {
                                        handles.push((handle_type, handle_name));
                                    }
                                }
                                "enum" => {
                                    //println!("Enum {:?}", element);
                                    // Should be handled elsewhere
                                }
                                "struct" => {
                                    //println!("Struct {:?}", element);
                                }

                                _ => {}
                            }

                        }
                    }
                }
            }
        }
    }

    // Write to file
    let mut content = String::new();

    // Handles
    content += "// Vulkan Handles\n";

    for (handle_type, handle_name) in handles {
        if handle_type.eq("VK_DEFINE_HANDLE") {
            content += "// VK_DEFINE_HANDLE\n";
            content += format!("pub type {} = *mut libc::c_void;\n\n", handle_name).as_str();
        } else if handle_type.eq("VK_DEFINE_NON_DISPATCHABLE_HANDLE") {
            content += "// VK_DEFINE_NON_DISPATCHABLE_HANDLE\n";
            content += format!("pub type {} = *mut libc::c_void;\n\n", handle_name).as_str();
        } else {
            panic!("Unknown handle type {} for name {}", handle_type, handle_name);
        }
    }

    // Vulkan constants
    content += "// Vulkan Constants\n";
    for (name, api_constant) in api_constants {
        if api_constant.variable_type.eq("uint32_t") {
            if let Ok(value) = api_constant.value.parse::<u32>() {
                content.push_str(format!("pub const {}: u32 = {};\n", api_constant.name.to_uppercase(), value).as_str());
            }
        }
        // TODO parse other constants
    }

    // Enums
    content += "\n";
    for vkenum in enums {
        content += format!("// Vulkan Enum - {} \n", vkenum.name).as_str();
        for (name, value) in vkenum.values {
            content += format!("pub const {}: i32 = {};\n", name, value).as_str();
        }
        content += "\n";
    }

    let mut file = File::create("src/bindings.rs").expect("Failed to create write file");
    file.write_all(content.as_ref()).expect("Failed to write to file");
}