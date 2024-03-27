pub mod ldap {

    mod settings;
    use crate::ldap::settings::settings::{Settings, load_settings};
    use crate::ldap::settings::settings::LdapSettings;
    use std::collections::HashMap;
    use ldap3::{LdapConnAsync, LdapConnSettings, Scope, SearchEntry, LdapError};
    use ldap3::adapters::{Adapter, EntriesOnly, PagedResults};


    pub struct MyLdapRequest {
        pub host: String,
        pub bind_dn: String,
        pub bind_pw: String,
        pub base: String,
        pub filter: String,
        pub attrs: Vec<String>
    }

    pub enum Mode {
        Display,        // mode affichage d'une fiche entry
        Search          // mode recherche d'une liste d'entries
    }

    pub struct Connexions {
        pub conf: Settings,
    }

    // Méthodes de Connexions
    impl Connexions {

        // constructeur
        pub fn new(filename: &str) -> Self {
            let conf = load_settings(filename);
            Self { conf }
        }

        // renvoie un objet MyLDAPRequest à partir d'un objet LdapSettings
        // si le mode est Search, les attributs utilisés sont les attrs_search de la conf toml et le add_filter est une recherche générique avec le char "*"
        // si le mode est Display, les attributs utilisés sont les attrs_display de la conf toml et le add_filter est une recherche précise sur un id
        pub fn request_for(&self, ldap_settings: &LdapSettings, mode: &Mode, add_filter_string: &String) -> MyLdapRequest {
            let filter = ldap_settings.filter.clone();
            let (attrs, final_filter) = match mode {
                Mode::Display => (ldap_settings.attrs_display.clone(), format!("(&(uid={}){})", add_filter_string, filter)),
                Mode::Search => (ldap_settings.attrs_search.clone(), format!("(&(displayName=*{}*){})", add_filter_string, filter))
            };
            MyLdapRequest {
                host: ldap_settings.host.clone(),
                bind_dn: ldap_settings.bind_dn.clone(),
                bind_pw: ldap_settings.bind_pw.clone(),
                base: ldap_settings.base.clone(),
                filter: final_filter,
                attrs
            }
        }

        // Execute la requête LDAP et renvoie un Résult contenant un vecteur de hashmaps
        pub async fn get_users(&self, ldap_request: &MyLdapRequest) -> Result<Vec<HashMap<String, Vec<String>>>, LdapError> {
                // Recherchr paginée pour la limite AD à 1000 entries
                // https://github.com/inejge/ldap3/blob/master/examples/search_adapted_paged_plus_sync.rs
                let adapters: Vec<Box<dyn Adapter<_, _>>> = vec![
                    Box::new(EntriesOnly::new()),
                    Box::new(PagedResults::new(1000)),
                ];

                let ldap_conn_settings: LdapConnSettings = LdapConnSettings::new().set_no_tls_verify(true);
                let (conn, mut ldap) = LdapConnAsync::with_settings(ldap_conn_settings, &ldap_request.host).await?;
                ldap3::drive!(conn);
                ldap.simple_bind(&ldap_request.bind_dn, &ldap_request.bind_pw).await?;


                let mut rs = ldap.streaming_search_with(
                    adapters,
                    &ldap_request.base,
                    Scope::Subtree,
                    &ldap_request.filter,
                    &ldap_request.attrs
                ).await?;
                
                let mut vec_res = Vec::new();
                while let Some(entry) = rs.next().await? {
                    let line: HashMap<String, Vec<String>> = SearchEntry::construct(entry).attrs;
                    vec_res.push(line);
                }
                
                ldap.unbind().await?;
                Ok(vec_res)
        }


        // renvoie une liste de users en mode Search
        pub async fn search(&self, add_filter: String) -> (Vec<HashMap<String, Vec<String>>>, Vec<HashMap<String, Vec<String>>>) {
           self.fetch_users(add_filter, Mode::Search).await
        }

        // renvoie une liste de users en mode Display (un user attendu, recherché par attr_id)
        pub async fn display(&self, add_filter: String) -> (Vec<HashMap<String, Vec<String>>>, Vec<HashMap<String, Vec<String>>>) {
            self.fetch_users(add_filter, Mode::Display).await
        }

        // Renvoie un tuple de vec de users
        pub async fn fetch_users(&self, add_filter: String, mode: Mode) -> (Vec<HashMap<String, Vec<String>>>, Vec<HashMap<String, Vec<String>>>) {
            let conf = &self.conf;
            let conf_ldap = &conf.ldap;
            let conf_ad = &conf.ad;
            let ldap_request = self.request_for(conf_ldap, &mode, &add_filter);
            let ad_request = self.request_for(conf_ad, &mode, &add_filter);
        
            let ldap_res: Result<Vec<HashMap<String, Vec<String>>>, LdapError> = self.get_users(&ldap_request).await;
            let ad_res: Result<Vec<HashMap<String, Vec<String>>>, LdapError> = self.get_users(&ad_request).await;

            let ldap_users: Vec<HashMap<String, Vec<String>>> = match ldap_res {
                Ok(users) => users,
                Err(_error) => vec![HashMap::from([("Erreur accès LDAP".to_string(), vec![])])],
            };

            let ad_users = match ad_res {
                Ok(users) => users,
                Err(_error) => vec![HashMap::from([("Erreur accès AD".to_string(), vec![])])],
            };
        
            (ldap_users, ad_users)        
        }
    }
 }