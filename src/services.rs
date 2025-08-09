//! Service registry for mDNS responder.
//!
//! This module manages the collection of services that are advertised
//! via mDNS, including their names, types, ports, and TXT records.

use dns_parser::{self, Name, QueryClass, RRData};
use multimap::MultiMap;
use rand::{thread_rng, Rng};
use std::collections::HashMap;
use std::slice;
use std::sync::{Arc, RwLock};

/// Type alias for DNS answer builder
pub type AnswerBuilder = dns_parser::Builder<dns_parser::Answers>;

/// Thread-safe collection of registered services
pub type Services = Arc<RwLock<ServicesInner>>;

/// The internal service registry.
///
/// Maintains multiple indices for efficient service lookup by ID, name, and type.
pub struct ServicesInner {
    hostname: Name<'static>,
    /// main index
    by_id: HashMap<usize, ServiceData>,
    /// maps to id
    by_type: MultiMap<Name<'static>, usize>,
    /// maps to id
    by_name: HashMap<Name<'static>, usize>,
}

impl ServicesInner {
    /// Creates a new service registry with the given hostname.
    pub fn new(hostname: String) -> Self {
        ServicesInner {
            hostname: Name::from_str(hostname)
                .expect("Invalid hostname format"),
            by_id: HashMap::new(),
            by_type: MultiMap::new(),
            by_name: HashMap::new(),
        }
    }

    /// Returns the hostname for this mDNS responder.
    pub fn get_hostname(&self) -> &Name<'static> {
        &self.hostname
    }

    /// Finds a service by its fully qualified domain name.
    pub fn find_by_name<'a>(&'a self, name: &'a Name<'a>) -> Option<&'a ServiceData> {
        self.by_name.get(name).and_then(|id| self.by_id.get(id))
    }

    /// Returns an iterator over all services of the given type.
    pub fn find_by_type<'a>(&'a self, ty: &'a Name<'a>) -> FindByType<'a> {
        let ids = self.by_type.get_vec(ty).map(|ids| ids.iter());

        FindByType {
            services: self,
            ids,
        }
    }

    /// Registers a new service and returns its unique ID.
    pub fn register(&mut self, svc: ServiceData) -> usize {
        let mut id = thread_rng().gen::<usize>();
        while self.by_id.contains_key(&id) {
            id = thread_rng().gen::<usize>();
        }

        self.by_type.insert(svc.typ.clone(), id);
        self.by_name.insert(svc.name.clone(), id);
        self.by_id.insert(id, svc);

        id
    }

    /// Unregisters a service by ID and returns its data.
    ///
    /// # Panics
    ///
    /// Panics if the service ID doesn't exist.
    pub fn unregister(&mut self, id: usize) -> ServiceData {
        let svc = self.by_id.remove(&id).expect("unknown service");

        if let Some(entries) = self.by_type.get_vec_mut(&svc.typ) {
            entries.retain(|&e| e != id);
        }

        let removed = self.by_name.remove(&svc.name);
        assert_eq!(removed, Some(id), "Service name index mismatch for id {id}");

        svc
    }
}

/// Returned by [`ServicesInner.find_by_type`](struct.ServicesInner.html#method.find_by_type)
pub struct FindByType<'a> {
    services: &'a ServicesInner,
    ids: Option<slice::Iter<'a, usize>>,
}

impl<'a> Iterator for FindByType<'a> {
    type Item = &'a ServiceData;

    fn next(&mut self) -> Option<Self::Item> {
        self.ids.as_mut()?
            .next()
            .and_then(|id| self.services.by_id.get(id))
    }
}

/// Data for a single mDNS service.
#[derive(Clone, Debug)]
pub struct ServiceData {
    pub name: Name<'static>,
    pub typ: Name<'static>,
    pub port: u16,
    pub txt: Vec<u8>,
}

impl ServiceData {
    /// Adds a PTR record for this service to the answer builder.
    pub fn add_ptr_rr(&self, builder: AnswerBuilder, ttl: u32) -> AnswerBuilder {
        builder.add_answer(
            &self.typ,
            QueryClass::IN,
            ttl,
            &RRData::PTR(self.name.clone()),
        )
    }

    /// Adds an SRV record for this service to the answer builder.
    pub fn add_srv_rr(&self, hostname: &Name, builder: AnswerBuilder, ttl: u32) -> AnswerBuilder {
        builder.add_answer(
            &self.name,
            QueryClass::IN,
            ttl,
            &RRData::SRV {
                priority: 0,
                weight: 0,
                port: self.port,
                target: hostname.clone(),
            },
        )
    }

    /// Adds a TXT record for this service to the answer builder.
    pub fn add_txt_rr(&self, builder: AnswerBuilder, ttl: u32) -> AnswerBuilder {
        builder.add_answer(&self.name, QueryClass::IN, ttl, &RRData::TXT(&self.txt))
    }
}
