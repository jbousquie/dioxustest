pub mod settings {
    use std::fs;
    use toml::Table;

    pub struct LdapSettings {
        pub host: String,
        pub bind_dn: String,
        pub bind_pw: String,
        pub base: String,
        pub filter: String,
        pub attr_id: String,
        pub attrs_search: Vec<String>,
        pub attrs_display: Vec<String>,
    }

    pub struct Settings {
        pub ldap: LdapSettings,
        pub ad: LdapSettings
    }

    // Renvoie une string à partir de la valeur d'un param du fichier toml
    fn string_from_settings(setting: &toml::Value) -> String {
        setting.as_str().unwrap().to_string()
    }

    // Renvoie  un vecteur de String à partir d'un param [string] du fichier toml
    fn vector_from_settings(setting: &toml::Value) -> Vec<String> {
        let attrs_toml = setting.as_array().unwrap();
        let mut attrs = Vec::new();
        for attr in attrs_toml.iter() {
            attrs.push(attr.as_str().unwrap().to_string());
        }
        attrs
    }

    // Renvoie un objet Settings à partir des valeurs lues dans le fichier de conf toml
    pub fn load_settings(settings_filename: &str) -> Settings {
        let settings_str = fs::read_to_string(settings_filename).unwrap();

        let config: &toml::map::Map<String, toml::Value> = &settings_str.parse::<Table>().unwrap();

        let ldap_conf = &config["ldap"];
        let ldap_attr_id = string_from_settings(&ldap_conf["ldap_attr_id"]);
        let mut ldap_attrs_search = vector_from_settings(&ldap_conf["ldap_attrs_search"]);
        ldap_attrs_search.push(ldap_attr_id.clone());

        let ldap_settings: LdapSettings = LdapSettings{
            host: string_from_settings(&ldap_conf["ldap_host"]),
            bind_dn: string_from_settings(&ldap_conf["ldap_bind_dn"]),
            bind_pw: string_from_settings(&ldap_conf["ldap_bind_pw"]),
            base: string_from_settings(&ldap_conf["ldap_base"]),
            filter: string_from_settings(&ldap_conf["ldap_filter"]),
            attr_id: ldap_attr_id,
            attrs_search: ldap_attrs_search,
            attrs_display: vector_from_settings(&ldap_conf["ldap_attrs_display"]),
        };

        let ad_conf = &config["ad"];
        let ad_attr_id = string_from_settings(&ad_conf["ad_attr_id"]);
        let mut ad_attrs_search = vector_from_settings(&ad_conf["ad_attrs_search"]);
        ad_attrs_search.push(ad_attr_id.clone());

        let ad_settings = LdapSettings {
            host: string_from_settings(&ad_conf["ad_host"]),
            bind_dn: string_from_settings(&ad_conf["ad_dn"]),
            bind_pw: string_from_settings(&ad_conf["ad_passwd"]),
            base: string_from_settings(&ad_conf["ad_base"]),
            filter: string_from_settings(&ad_conf["ad_filter"]),
            attr_id: ad_attr_id,
            attrs_search: ad_attrs_search,
            attrs_display: vector_from_settings(&ad_conf["ad_attrs_display"]),
        };

        let settings = Settings {
            ldap: ldap_settings,
            ad: ad_settings
        };

        settings
    }
}