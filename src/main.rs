#![allow(non_snake_case)]
use std::collections::HashMap;

use dioxus::{
    desktop::{tao::dpi::PhysicalSize, Config, LogicalSize, WindowBuilder}, prelude::*
};
use searchuser::ldap::Connexions;


// Colors
static LDAP_COL1: &str = "rgba(95, 158, 160, 0.4)";
static LDAP_COL2: &str = "rgba(175, 238, 238, 0.4)";
static AD_COL1: &str =  "rgba(158, 158, 100, 0.4)";
static AD_COL2: &str = "rgba(238, 238, 100, 0.4)";

pub struct Results {
    ldap_res: Vec<Vec<String>>,
    ad_res: Vec<Vec<String>>,
}

fn main() {
    
    LaunchBuilder::desktop()
        .with_cfg(
            Config::new()
                .with_window(
                    WindowBuilder::new()
                        .with_title("Dioxus Test")
                        .with_inner_size(LogicalSize::new(1280.0, 900.0))
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
                class: "textInput",
                r#type: "text",
                 //value: "{name}",
                placeholder: "3 caractères minimum",
                onchange:  move |event| { 
                    name_signal.set(event.value());
                }
            }
            EntryList { signal: name_signal } 
        }
    }
}

pub async fn search_results(filter: String) -> Results {

    let settings_filename = "./conf.toml";
    let  con = Connexions::new(settings_filename);
    let ldap_attrs = &con.conf.ldap.attrs_search;

    let (ldap_res, ad_res) = con.search(filter).await;
    let ad_attrs = &con.conf.ad.attrs_search;   
    let res_ldap = format_data(&ldap_attrs, ldap_res);
    let res_ad = format_data(&ad_attrs, ad_res);

    let res = Results {
        ldap_res: res_ldap,
        ad_res: res_ad,
    };
    res
}




// exemple : https://dioxuslabs.com/learn/0.5/guide/data_fetching
#[component]
fn EntryList(signal: Signal<String>) -> Element {
   
    let res = use_resource(move || { 
        let filter = signal();
        search_results(filter)
    });

    match &*res.read_unchecked() {
        Some(res) => {
            let list_ldap = res.ldap_res.clone();
            let list_ad = res.ad_res.clone();

            let scale = 10; // taille en px d'un caractère
            let ldap_lengths = field_lengths(&list_ldap);
            let ad_lengths = field_lengths(&list_ad);

            rsx! {
                div {
                    class: "dataArrays",
                    DisplayList{ list: list_ldap, lengths: ldap_lengths, scale: scale, color1: LDAP_COL1.to_string(), color2: LDAP_COL2.to_string() },
                    DisplayList{ list: list_ad, lengths: ad_lengths, scale: scale, color1: AD_COL1.to_string(), color2: AD_COL2.to_string() }
                }
            }
        },

        None => {
            rsx! { 
                div {
                    class: "msg",
                    "Recherche en cours ..."
                }
            }
        },

    } 
}

#[derive(PartialEq, Props, Clone)]
struct DisplayListProps {
    list: Vec<Vec<String>>,
    lengths: Vec<usize>,
    scale: usize,
    color1: String,
    color2: String,
}
#[component]
fn DisplayList(props: DisplayListProps) -> Element {
    let mut odd = false;
    let color1 = props.color1;
    let color2 = props.color2;
    let mut header = true;
    rsx! {
        div { 
            class: "resultCol",
            width: {
                let mut total_width = 0;
                for i in 0..props.lengths.len() - 1 { // on ne prend pas en compte le dernier champ "uid" ajouté
                    total_width += props.lengths[i];
                }
                let upscaled = (props.scale as f64 * 1.05).round() as usize;
                let div_width = (upscaled * total_width).to_string() + "px";
                div_width.clone()
            },
            for line in props.list {
                div {
                    class: {
                        if header {
                            header = false;
                            "header".to_string()
                        }
                        else {
                            "ldap_line".to_string()
                        }
                    },
                    background_color: {
                        odd = !odd; 
                        if odd { color1.clone() } else { color2.clone() }
                    } ,
                    for  i in 0..line.len() - 1 {    // on ne prend pas en compte le dernier champ "uid" ajouté
                        div {
                            class: "ldap_field",
                            width: {
                                (props.lengths[i] * props.scale).to_string() + "px"
                            },
                            { line[i].clone() }
                        }
                    }
                }
            }
        }
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
                    let empty = String::from("<vide>");
                    values_line.push(empty);
                } 
            }
            lines.push(values_line);
        }
    }
    lines
}

