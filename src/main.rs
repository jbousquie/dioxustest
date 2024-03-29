#![allow(non_snake_case)]
use std::collections::HashMap;

use dioxus::{
    desktop::{Config, LogicalSize, WindowBuilder}, prelude::*
};
use searchuser::ldap::Connexions;


// Colors
static ldap_col1: &str = "rgba(95, 158, 160, 0.4)";
static ldap_col2: &str = "rgba(175, 238, 238, 0.4)";
static ad_col1: &str =  "rgba(158, 158, 100, 0.4)";
static ad_col2: &str = "rgba(238, 238, 100, 0.4)";



fn main() {
    
    LaunchBuilder::desktop()
        .with_cfg(
            Config::new()
                .with_window(
                    WindowBuilder::new()
                        .with_title("Appli de test")
                        .with_inner_size(LogicalSize::new(1280, 900))
                        .with_maximized(false),
                )
                .with_custom_head(r#"<link rel="stylesheet" href="default.css">"#.to_string()),
        )
        .launch(App);
}


// https://github.com/DioxusLabs/dioxus/blob/master/examples/weather_app.rs
fn App() -> Element {
    let mut name_signal = use_signal(|| "---".to_string());
    rsx! {
        div {
            input {
                r#type: "text",
                 //value: "{name}",
                placeholder: "3 caractères minimum",
                autofocus: true,
                onchange:  move |event| { 
                    name_signal.set(event.value());
                }
            }
            EntryList { signal: name_signal } 
        }
    }
}

pub async fn search_results(filter: String) -> Vec<Vec<String>> {

    let settings_filename = "./conf.toml";
    let  con = Connexions::new(settings_filename);
    let ldap_attrs = &con.conf.ldap.attrs_search;

    let (ldap_res, ad_res) = con.search(filter).await;
    //let ad_attrs = &con.conf.ad.attrs_search;   
    let res_ldap = format_data(&ldap_attrs, ldap_res);
    //let res_ad = format_data(&ad_attrs, ad_res);

    res_ldap
}




// exemple : https://dioxuslabs.com/learn/0.5/guide/data_fetching
#[component]
fn EntryList(signal: Signal<String>) -> Element {
   
    let res_ldap = use_resource(move || { 
        let filter = signal();
        search_results(filter)
    });


    let mut odd = false;
    let mut i = 0;
    match &*res_ldap.read_unchecked() {
        Some(list) => {
            let lengths = field_lengths(list);
            let field_nb = lengths.len();

            rsx! {
                div { 
                    for line in list {
                        div {
                            background_color: {
                                odd = !odd; 
                                if odd {ldap_col1} else {ldap_col2}
                            } ,
                            for field in line {
                                div {
                                    class: "ldap_field",
                                    width: {
                                        let l = lengths[i];
                                        i+= 1;
                                        if i == field_nb {
                                            i = 0;
                                        }
                                        (l * 10).to_string() + "px"
                                    },
                                    { field.clone() }
                                }
                            }
                        }
                    }
                }
            }
        },

        None => {
            rsx! { "Recherche en cours ..."}
        },

    } 
}

// Renvoie un tableau de taille de champ pour la liste de résultats passée, en fonction de la longueur du plus grand mot de chaque champ
fn field_lengths(res: &Vec<Vec<String>>) -> Vec<usize> {
    let field_number = res[0].len();
    let mut lengths = vec!(0; field_number);
    for line in res {
        for i in 0..field_number {
            if line[i].len() > lengths[i] {
                lengths[i] = line[i].chars().count();
            }
        }
    }
    lengths
}

fn format_data(attrs: &Vec<String>, res: Vec<HashMap<String, Vec<String>>>) -> Vec<Vec<String>> {
    let mut lines = Vec::new();
    lines.push(attrs.clone());
    if res.len() > 0 {
        for line in res.into_iter() {
            let mut values_line = vec!();
            for attr in attrs.into_iter() {
                if line.contains_key(attr) {
                    let vct = &line[attr];
                    let mut vals = vct[0].clone();
                    let l = vct.len();
                    if l > 1 {
                        for i in 1..l {
                            let val = &vct[i];
                            vals = vals + "\n" + val;
                        }
                    }
                    values_line.push(vals);
                } 
                else {
                    let empty = String::from("");
                    values_line.push(empty);
                } 
            }
            lines.push(values_line);
        }
    }
    lines
}

