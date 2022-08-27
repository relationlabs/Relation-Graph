#![cfg_attr(not(feature = "std"), no_std)]

use pallet_timestamp as timestamp;

use std::convert::{Infallible, TryFrom, TryInto};
use std::io::{BufRead, BufReader, Write};
use std::vec::IntoIter;

use frame_support::pallet_prelude::*;

use crate::error::UnwrapInfallible;
use crate::io::{DatasetFormat, DatasetParser, GraphFormat, GraphParser};
use crate::model::*;
use crate::sparql::{
    EvaluationError,
    Query,
    QueryOptions,
    QueryResults,
    QueryResultsFormat,
    Update,
    UpdateOptions,
};
use crate::store::{
    ReadableEncodedStore,
    WritableEncodedStore,
    model::StrHash,
    codec::{EncodedQuad, EncodedTerm},
    numeric_encoder::{
        Decoder,
        ReadEncoder,
        StrContainer,
        StrEncodingAware,
        StrId,
        StrLookup,
        WriteEncoder,
    },
};

pub use pallet::*;

mod error;
mod io;
mod model;
mod sparql;
mod store;
mod ogm;

type IoError = std::io::Error;

const PREFIX: &str = "
      prefix : <http://relationlabs.ai/entity/>
      prefix rdf: <http://www.w3.org/1999/02/22-rdf-syntax-ns#>
      prefix rdfs: <http://www.w3.org/2000/01/rdf-schema#>
      prefix xsd: <http://www.w3.org/2001/XMLSchema#>
";
const GRAPH_NAME_ACL: &str = "http://relationlabs.ai/acl/";

#[frame_support::pallet]
pub mod pallet {
    use frame_support::pallet_prelude::*;
    use frame_support::storage::Key;
    use frame_system::pallet_prelude::*;

    use super::timestamp;

    /// Configure the pallet by specifying the parameters and types on which it depends.
    #[pallet::config]
    pub trait Config: frame_system::Config + timestamp::Config {
        /// Because this pallet emits events, it depends on the runtime's definition of an event.
        type Event: From<Event<Self>> + IsType<<Self as frame_system::Config>::Event>;

        /// The maximum length a name may be.
        #[pallet::constant]
        type MaxValueLength: Get<u32>;
    }

    #[pallet::pallet]
    #[pallet::generate_store(pub (super) trait Store)]
    pub struct Pallet<T>(PhantomData<T>);

    // ID to Str
    #[pallet::storage]
    #[pallet::getter(fn id2str)]
    pub type Id2StrStore<T: Config> = StorageMap<_, Blake2_128Concat, u128, BoundedVec<u8, T::MaxValueLength>>;

    // Graph names (only store name as key, value is useless)
    #[pallet::storage]
    #[pallet::getter(fn graphs)]
    pub type GraphNameStore<T: Config> = StorageMap<_, Blake2_128Concat, BoundedVec<u8, T::MaxValueLength>, bool>;

    // Default GraphStore
    #[pallet::storage]
    #[pallet::getter(fn default_spo)]
    pub type DefaultSpoStore<T: Config> = StorageNMap<
        _,
        (
            Key<Blake2_128Concat, BoundedVec<u8, T::MaxValueLength>>, // s (subject)
            Key<Blake2_128Concat, BoundedVec<u8, T::MaxValueLength>>, // p (predicate)
            Key<Blake2_128Concat, BoundedVec<u8, T::MaxValueLength>>, // o (object)
        ),
        //BoundedVec<u8, T::MaxValueLength>, // value TODO: value is useless, just use a fixed bool or u8?
        bool,
        OptionQuery, // ValueQuery
    >;

    #[pallet::storage]
    #[pallet::getter(fn default_pos)]
    pub type DefaultPosStore<T: Config> = StorageNMap<
        _,
        (
            Key<Blake2_128Concat, BoundedVec<u8, T::MaxValueLength>>, // p
            Key<Blake2_128Concat, BoundedVec<u8, T::MaxValueLength>>, // o
            Key<Blake2_128Concat, BoundedVec<u8, T::MaxValueLength>>, // s
        ),
        bool,
        OptionQuery,
    >;

    #[pallet::storage]
    #[pallet::getter(fn default_osp)]
    pub type DefaultOspStore<T: Config> = StorageNMap<
        _,
        (
            Key<Blake2_128Concat, BoundedVec<u8, T::MaxValueLength>>, // o
            Key<Blake2_128Concat, BoundedVec<u8, T::MaxValueLength>>, // s
            Key<Blake2_128Concat, BoundedVec<u8, T::MaxValueLength>>, // p
        ),
        bool,
        OptionQuery,
    >;

    // Named GraphStore

    #[pallet::storage]
    #[pallet::getter(fn gspo)]
    pub type GspoStore<T: Config> = StorageNMap<
        _,
        (
            Key<Blake2_128Concat, BoundedVec<u8, T::MaxValueLength>>, // g (graph)
            Key<Blake2_128Concat, BoundedVec<u8, T::MaxValueLength>>, // s (subject)
            Key<Blake2_128Concat, BoundedVec<u8, T::MaxValueLength>>, // p (predicate)
            Key<Blake2_128Concat, BoundedVec<u8, T::MaxValueLength>>, // o (object)
        ),
        bool,
        OptionQuery,
    >;

    #[pallet::storage]
    #[pallet::getter(fn gpos)]
    pub type GposStore<T: Config> = StorageNMap<
        _,
        (
            Key<Blake2_128Concat, BoundedVec<u8, T::MaxValueLength>>, // g
            Key<Blake2_128Concat, BoundedVec<u8, T::MaxValueLength>>, // p
            Key<Blake2_128Concat, BoundedVec<u8, T::MaxValueLength>>, // o
            Key<Blake2_128Concat, BoundedVec<u8, T::MaxValueLength>>, // s
        ),
        bool,
        OptionQuery,
    >;

    #[pallet::storage]
    #[pallet::getter(fn gosp)]
    pub type GospStore<T: Config> = StorageNMap<
        _,
        (
            Key<Blake2_128Concat, BoundedVec<u8, T::MaxValueLength>>, // g
            Key<Blake2_128Concat, BoundedVec<u8, T::MaxValueLength>>, // o
            Key<Blake2_128Concat, BoundedVec<u8, T::MaxValueLength>>, // s
            Key<Blake2_128Concat, BoundedVec<u8, T::MaxValueLength>>, // p
        ),
        bool,
        OptionQuery,
    >;

    #[pallet::storage]
    #[pallet::getter(fn spog)]
    pub type SpogStore<T: Config> = StorageNMap<
        _,
        (
            Key<Blake2_128Concat, BoundedVec<u8, T::MaxValueLength>>, // s
            Key<Blake2_128Concat, BoundedVec<u8, T::MaxValueLength>>, // p
            Key<Blake2_128Concat, BoundedVec<u8, T::MaxValueLength>>, // o
            Key<Blake2_128Concat, BoundedVec<u8, T::MaxValueLength>>, // g
        ),
        bool,
        OptionQuery,
    >;

    #[pallet::storage]
    #[pallet::getter(fn posg)]
    pub type PosgStore<T: Config> = StorageNMap<
        _,
        (
            Key<Blake2_128Concat, BoundedVec<u8, T::MaxValueLength>>, // p
            Key<Blake2_128Concat, BoundedVec<u8, T::MaxValueLength>>, // o
            Key<Blake2_128Concat, BoundedVec<u8, T::MaxValueLength>>, // s
            Key<Blake2_128Concat, BoundedVec<u8, T::MaxValueLength>>, // g
        ),
        bool,
        OptionQuery,
    >;

    #[pallet::storage]
    #[pallet::getter(fn ospg)]
    pub type OspgStore<T: Config> = StorageNMap<
        _,
        (
            Key<Blake2_128Concat, BoundedVec<u8, T::MaxValueLength>>, // o
            Key<Blake2_128Concat, BoundedVec<u8, T::MaxValueLength>>, // s
            Key<Blake2_128Concat, BoundedVec<u8, T::MaxValueLength>>, // p
            Key<Blake2_128Concat, BoundedVec<u8, T::MaxValueLength>>, // g
        ),
        bool,
        OptionQuery,
    >;

    // Pallets use events to inform users when important changes are made.
    // https://docs.substrate.io/v3/runtime/events-and-errors
    #[pallet::event]
    #[pallet::generate_deposit(pub (super) fn deposit_event)]
    pub enum Event<T: Config> {
        /// parameters. [who]
        DataInitialized(T::AccountId),

        /// parameters. [who]
        DataUpdate(T::AccountId),
    }

    // Errors inform users that something went wrong.
    #[pallet::error]
    pub enum Error<T> {
        /// Error names should be descriptive.
        NoneValue,
        /// Errors should have helpful documentation associated with them.
        StorageOverflow,
    }

    // You can implement the [`Hooks`] trait to define some logic
    // that should be exectued regularly in some context.
    #[pallet::hooks]
    impl<T: Config> Hooks<BlockNumberFor<T>> for Pallet<T> {
        // `on_initialize` is executed at the beginning of the block before any extrinsics are
        // dispatched.
        //
        // This function must return the weight consumed by `on_initialize` and `on_finalize`.
        fn on_initialize(_n: T::BlockNumber) -> Weight {
            // Anything that needs to be done at the start of the block.
            0
        }
    }

    // Dispatchable functions allows users to interact with the pallet and invoke state changes.
    // These functions materialize as "extrinsics", which are often compared to transactions.
    // Dispatchable functions must be annotated with a weight and must return a DispatchResult.
    #[pallet::call]
    impl<T: Config> Pallet<T> {
        #[pallet::weight(10_000 + T::DbWeight::get().writes(1))]
        pub fn init_db(origin: OriginFor<T>) -> DispatchResult {
            let who = ensure_signed(origin)?;
            Self::init_data();
            Self::deposit_event(Event::DataInitialized(who));
            Ok(())
        }

        /// Execute sparql update
        #[pallet::weight(10_000 + T::DbWeight::get().writes(1))]
        pub fn sparql_update(origin: OriginFor<T>, update: Vec<u8>) -> DispatchResult {
            let who = ensure_signed(origin)?;
            let update = std::str::from_utf8(&update).unwrap();
            Self::execute_update(update);
            Self::deposit_event(Event::DataUpdate(who));
            Ok(())
        }
    }
}

//****************************
/// Impl pallet
//****************************
impl<T: Config> Pallet<T> {

    pub fn init_data() {
        let graph_store = GraphStore::<T>::new();

        let acl_data = include_bytes!("../../../data/relation_acl.ttl");
        let acl_graph = GraphName::from(NamedNode::new(GRAPH_NAME_ACL).unwrap());
        graph_store.load_graph(BufReader::new(&acl_data[..]),
                               GraphFormat::Turtle,
                               &acl_graph,
                               None).unwrap();

        // default graph
        let data = include_bytes!("../../../data/relation_samples.ttl");
        graph_store.load_graph(BufReader::new(&data[..]),
                               GraphFormat::Turtle,
                               &GraphName::DefaultGraph,
                               None).unwrap();

    }

    pub fn execute_query<S: AsRef<str>>(query: S) -> String {
        let now = <timestamp::Pallet<T>>::get();
        println!("now: {:?}", now);
        let graph_store = GraphStore::<T>::new();
        let sparql = format!("
              {}
              {}
            ", PREFIX, query.as_ref());
        println!("sparql_query: {:?}", sparql);
        let query = Query::parse(&sparql, None).unwrap();
        let query_result = graph_store.query(query).unwrap();
        let mut buffer = Vec::default();
        query_result.write(&mut buffer, QueryResultsFormat::Json).unwrap();
        String::from_utf8_lossy(&buffer[..]).to_string()
    }

    pub fn execute_update<S: AsRef<str>>(update: S) {
        let graph_store = GraphStore::<T>::new();
        let sparql = format!("
              {}
              {}
            ", PREFIX, update.as_ref());
        println!("sparql_update: {}", sparql);
        let update = Update::parse(&sparql, None).unwrap();
        let result = graph_store.update(update);
        println!("sparql_update result: {:?}", result);
    }
}

//****************************
/// Impl graph storage
//****************************
#[derive(Debug, Clone)]
struct GraphStore<T> {
    _p: PhantomData<T>,
}

impl<T: Config> GraphStore<T> {
    pub fn new() -> Self {
        Self {
            _p: PhantomData,
        }
    }

    /// Executes a [SPARQL 1.1 query](https://www.w3.org/TR/sparql11-query/).
    pub fn query(
        &self,
        query: impl TryInto<Query, Error=impl Into<EvaluationError>>,
    ) -> Result<QueryResults, EvaluationError> {
        self.query_with_options(query, QueryOptions::default())
    }

    /// Executes a [SPARQL 1.1 query](https://www.w3.org/TR/sparql11-query/) with some options.
    pub fn query_with_options(
        &self,
        query: impl TryInto<Query, Error=impl Into<EvaluationError>>,
        options: QueryOptions,
    ) -> Result<QueryResults, EvaluationError> {
        sparql::evaluate_query(self.clone(), query, options)
    }

    /// Executes a [SPARQL 1.1 update](https://www.w3.org/TR/sparql11-update/).
    pub fn update(
        &self,
        update: impl TryInto<Update, Error = impl Into<EvaluationError>>,
    ) -> Result<(), EvaluationError> {
        self.update_with_options(update, UpdateOptions::default())
    }

    /// Executes a [SPARQL 1.1 update](https://www.w3.org/TR/sparql11-update/) with some options.
    pub fn update_with_options(
        &self,
        update: impl TryInto<Update, Error = impl Into<EvaluationError>>,
        options: UpdateOptions,
    ) -> Result<(), EvaluationError> {
        sparql::evaluate_update(
            self.clone(),
            &mut &*self,
            update.try_into().map_err(|e| e.into())?,
            options,
        )
    }

    /// Adds a quad to this store.
    #[allow(clippy::needless_pass_by_value)]
    pub fn insert_quad(&self, quad: impl Into<Quad>) {
        let mut this = self;
        let quad = this.encode_quad(quad.into().as_ref()).unwrap_infallible();
        this.insert_encoded(&quad).unwrap_infallible();
    }

    /// Removes a quad from this store.
    pub fn remove_quad<'a>(&self, quad: impl Into<QuadRef<'a>>){
        if let Ok(Some(quad))= self.get_encoded_quad(quad.into()){
            let mut this = self;
            this.remove_encoded(&quad).unwrap_infallible();
        }
    }

    /// Loads a graph (e.g. triples)from reader into the store.
    pub fn load_graph<'a>(
        &self,
        reader: impl BufRead,
        format: GraphFormat,
        to_graph_name: impl Into<GraphNameRef<'a>>,
        base_iri: Option<&str>,
    ) -> Result<(), IoError> {
        let mut store = self;
        store::load_graph(&mut store, reader, format, to_graph_name.into(), base_iri)?;
        Ok(())
    }

    /// Dumps a store graph into a writer.
    pub fn dump_graph<'a>(
        &self,
        writer: impl Write,
        format: GraphFormat,
        from_graph_name: impl Into<GraphNameRef<'a>>,
    ) -> Result<(), IoError> {
        let iter=self.quads_for_pattern(None, None, None, Some(from_graph_name.into()))
            .map(|q| Ok(q.into()));
        store::dump_graph(iter, writer, format)
    }

    /// Returns all the store named graphs
    pub fn named_graphs(&self) -> GraphNameIter<T> {
        GraphNameIter {
            iter: self.encoded_named_graphs(),
            store: self.clone(),
        }
    }

    /// Checks if the store contains a given graph
    pub fn contains_named_graph<'a>(
        &self,
        graph_name: impl Into<NamedOrBlankNodeRef<'a>>,
    ) -> bool {
        if let Some(graph_name) = self
            .get_encoded_named_or_blank_node(graph_name.into()).unwrap_infallible()
        {
            self.contains_encoded_named_graph(graph_name).unwrap_infallible()
        } else {
            false
        }
    }

    /// Inserts a graph into this store
    pub fn insert_named_graph(&self, graph_name: impl Into<NamedOrBlankNode>) {
        let mut this = self;
        let graph_name = this
            .encode_named_or_blank_node(graph_name.into().as_ref())
            .unwrap_infallible();
        this.insert_encoded_named_graph(graph_name)
            .unwrap_infallible()
    }

    /// Removes a graph from this store.
    pub fn remove_named_graph<'a>(&self, graph_name: impl Into<NamedOrBlankNodeRef<'a>>) {
        if let Some(graph_name) = self
            .get_encoded_named_or_blank_node(graph_name.into())
            .unwrap_infallible()
        {
            let mut this = self;
            this.remove_encoded_named_graph(graph_name)
                .unwrap_infallible()
        }
    }

    /// Clears a graph from this store.
    pub fn clear_graph<'a>(&self, graph_name: impl Into<GraphNameRef<'a>>) {
        if let Some(graph_name) = self
            .get_encoded_graph_name(graph_name.into())
            .unwrap_infallible()
        {
            let mut this = self;
            this.clear_encoded_graph(graph_name).unwrap_infallible()
        }
    }

    /// Clears the store.
    pub fn clear_all(&self) {
        let mut this = self;
        this.clear().unwrap_infallible()
    }

    /// Retrieves quads with a filter on each quad component (used by dump_graph & ReadableEncodedStore)
    fn quads_for_pattern(
        &self,
        subject: Option<NamedOrBlankNodeRef<'_>>,
        predicate: Option<NamedNodeRef<'_>>,
        object: Option<TermRef<'_>>,
        graph_name: Option<GraphNameRef<'_>>,
    ) -> QuadIter<T>
    {
        if let Some((subject, predicate, object, graph_name)) =
        store::get_encoded_quad_pattern(self, subject, predicate, object, graph_name).unwrap_infallible()
        {
            let iter = self.encoded_quads_for_pattern(subject, predicate, object, graph_name);
            QuadIter::new(iter, self.clone())
        } else {
            QuadIter::empty()
        }
    }

    /// Query quads for the given pattern
    fn encoded_quads_for_pattern(
        &self,
        subject: Option<EncodedTerm>,
        predicate: Option<EncodedTerm>,
        object: Option<EncodedTerm>,
        graph_name: Option<EncodedTerm>,
    ) -> EncodedQuadsIter
    {
        match subject {
            Some(subject) => match predicate {
                // spog, spo, spg, sp
                Some(predicate) => match object {
                    Some(object) => match graph_name {
                        // spog
                        Some(graph_name) => {
                            self.quads_for_subject_predicate_object_graph(subject, predicate, object, graph_name)
                        }
                        // spo
                        None => self.quads_for_subject_predicate_object(subject, predicate, object),
                    },
                    None => match graph_name {
                        // spg
                        Some(graph_name) => {
                            self.quads_for_subject_predicate_graph(subject, predicate, graph_name)
                        }
                        // sp
                        None => self.quads_for_subject_predicate(subject, predicate),
                    },
                },
                // sog, so, sg, s
                None => match object {
                    Some(object) => match graph_name {
                        // sog
                        Some(graph_name) => {
                            self.quads_for_subject_object_graph(subject, object, graph_name)
                        }
                        // so
                        None => self.quads_for_subject_object(subject, object),
                    },
                    None => match graph_name {
                        // sg
                        Some(graph_name) => self.quads_for_subject_graph(subject, graph_name),
                        // s
                        None => self.quads_for_subject(subject),
                    },
                },
            },
            None => match predicate {
                // pog, po, pg, p
                Some(predicate) => match object {
                    Some(object) => match graph_name {
                        // pog
                        Some(graph_name) => {
                            self.quads_for_predicate_object_graph(predicate, object, graph_name)
                        }
                        // po
                        None => self.quads_for_predicate_object(predicate, object),
                    },
                    None => match graph_name {
                        // pg
                        Some(graph_name) => self.quads_for_predicate_graph(predicate, graph_name),
                        // p
                        None => self.quads_for_predicate(predicate),
                    },
                },
                // og,o,g,all
                None => match object {
                    Some(object) => match graph_name {
                        // og
                        Some(graph_name) => self.quads_for_object_graph(object, graph_name),
                        // o
                        None => self.quads_for_object(object),
                    },
                    None => match graph_name {
                        // g
                        Some(graph_name) => self.quads_for_graph(graph_name),
                        // all
                        None => self.quads(),
                    },
                },
            },
        }
    }

    // Step1 pattern: spog
    fn quads_for_subject_predicate_object_graph(
        &self,
        subject: EncodedTerm,
        predicate: EncodedTerm,
        object: EncodedTerm,
        graph_name: EncodedTerm,
    ) -> EncodedQuadsIter {
        let quad = EncodedQuad::new(subject, predicate, object, graph_name);
        let quads = if self.contains_encoded_quad(&quad) {
            vec![quad]
        } else {
            vec![]
        };
        EncodedQuadsIter::new(EncodedQuadIter { iter: quads.into_iter() })
    }

    // Check quad(s,p,o,g)
    fn contains_encoded_quad(&self, quad: &EncodedQuad) -> bool {
        let s = quad.subject.to_bounded_vec();
        let p = quad.predicate.to_bounded_vec();
        let o = quad.object.to_bounded_vec();
        if quad.graph_name.is_default_graph() {
            <DefaultSpoStore<T>>::contains_key((s, p, o))
        } else {
            let g = quad.graph_name.to_bounded_vec();
            <SpogStore<T>>::contains_key((s, p, o, g))
        }
    }

    // Step2 pattern: spo
    fn quads_for_subject_predicate_object(
        &self,
        subject: EncodedTerm,
        predicate: EncodedTerm,
        object: EncodedTerm,
    ) -> EncodedQuadsIter {
        let s = subject.to_bounded_vec();
        let p = predicate.to_bounded_vec();
        let o = object.to_bounded_vec();

        let default_graph = vec![EncodedQuad::new(subject, predicate, object, EncodedTerm::DefaultGraph)];
        let named_graph_iter = <SpogStore<T>>::iter_key_prefix((s, p, o, ))
            .map(|g| { // v is a BoundedVec of graph name
                let graph_name = EncodedTerm::from_bytes(g.as_ref());
                EncodedQuad::new(subject, predicate, object, graph_name)
            }).collect::<Vec<_>>().into_iter();

        EncodedQuadsIter::pair(
            EncodedQuadIter { iter: default_graph.into_iter() },
            EncodedQuadIter { iter: named_graph_iter },
        )
    }

    // Step3 pattern: spg
    fn quads_for_subject_predicate_graph(
        &self,
        subject: EncodedTerm,
        predicate: EncodedTerm,
        graph_name: EncodedTerm,
    ) -> EncodedQuadsIter {
        let s = subject.to_bounded_vec();
        let p = predicate.to_bounded_vec();

        let iter = if graph_name.is_default_graph() {
            <DefaultSpoStore<T>>::iter_key_prefix((s, p, ))
                .map(|o| {
                    let object = EncodedTerm::from_bytes(o.as_ref());
                    EncodedQuad::new(subject, predicate, object, graph_name)
                }).collect::<Vec<_>>().into_iter()
        } else {
            let g = graph_name.to_bounded_vec();
            <GspoStore<T>>::iter_key_prefix((g, s, p, ))
                .map(|o| {
                    let object = EncodedTerm::from_bytes(o.as_ref());
                    EncodedQuad::new(subject, predicate, object, graph_name)
                }).collect::<Vec<_>>().into_iter()
        };
        EncodedQuadsIter::new(EncodedQuadIter { iter })
    }

    // Step4 pattern: sp
    fn quads_for_subject_predicate(
        &self,
        subject: EncodedTerm,
        predicate: EncodedTerm,
    ) -> EncodedQuadsIter {
        let s = subject.to_bounded_vec();
        let p = predicate.to_bounded_vec();

        let default_graph_iter =
            <DefaultSpoStore<T>>::iter_key_prefix((s.clone(), p.clone(), ))
                .map(|o| {
                    let object = EncodedTerm::from_bytes(o.as_ref());
                    EncodedQuad::new(subject, predicate, object, EncodedTerm::DefaultGraph)
                }).collect::<Vec<_>>().into_iter();

        let named_graph_iter =
            <SpogStore<T>>::iter_key_prefix((s, p, ))
                .map(|(o, g)| {
                    let object = EncodedTerm::from_bytes(o.as_ref());
                    let graph_name = EncodedTerm::from_bytes(g.as_ref());
                    EncodedQuad::new(subject, predicate, object, graph_name)
                }).collect::<Vec<_>>().into_iter();

        EncodedQuadsIter::pair(
            EncodedQuadIter { iter: default_graph_iter },
            EncodedQuadIter { iter: named_graph_iter },
        )
    }

    // Step5 pattern: sog
    fn quads_for_subject_object_graph(
        &self,
        subject: EncodedTerm,
        object: EncodedTerm,
        graph_name: EncodedTerm,
    ) -> EncodedQuadsIter {
        let s = subject.to_bounded_vec();
        let o = object.to_bounded_vec();

        let iter = if graph_name.is_default_graph() {
            <DefaultOspStore<T>>::iter_key_prefix((o, s, ))
                .map(|p| {
                    let predicate = EncodedTerm::from_bytes(p.as_ref());
                    EncodedQuad::new(subject, predicate, object, graph_name)
                }).collect::<Vec<_>>().into_iter()
        } else {
            let g = graph_name.to_bounded_vec();
            <GospStore<T>>::iter_key_prefix((g, o, s, ))
                .map(|p| {
                    let predicate = EncodedTerm::from_bytes(p.as_ref());
                    EncodedQuad::new(subject, predicate, object, graph_name)
                }).collect::<Vec<_>>().into_iter()
        };
        EncodedQuadsIter::new(EncodedQuadIter { iter })
    }

    // Step6 pattern: so
    fn quads_for_subject_object(
        &self,
        subject: EncodedTerm,
        object: EncodedTerm,
    ) -> EncodedQuadsIter {
        let s = subject.to_bounded_vec();
        let o = object.to_bounded_vec();
        let default_graph_iter =
            <DefaultOspStore<T>>::iter_key_prefix((o.clone(), s.clone(), ))
                .map(|p| {
                    let predicate = EncodedTerm::from_bytes(p.as_ref());
                    EncodedQuad::new(subject, predicate, object, EncodedTerm::DefaultGraph)
                }).collect::<Vec<_>>().into_iter();

        let named_graph_iter =
            <OspgStore<T>>::iter_key_prefix((o, s, ))
                .map(|(p, g)| {
                    let predicate = EncodedTerm::from_bytes(p.as_ref());
                    let graph_name = EncodedTerm::from_bytes(g.as_ref());
                    EncodedQuad::new(subject, predicate, object, graph_name)
                }).collect::<Vec<_>>().into_iter();

        EncodedQuadsIter::pair(
            EncodedQuadIter { iter: default_graph_iter },
            EncodedQuadIter { iter: named_graph_iter },
        )
    }

    // Step7 pattern: sg
    fn quads_for_subject_graph(
        &self,
        subject: EncodedTerm,
        graph_name: EncodedTerm,
    ) -> EncodedQuadsIter {
        let s = subject.to_bounded_vec();

        let iter = if graph_name.is_default_graph() {
            <DefaultSpoStore<T>>::iter_key_prefix((s, ))
                .map(|(p, o)| {
                    let predicate = EncodedTerm::from_bytes(p.as_ref());
                    let object = EncodedTerm::from_bytes(o.as_ref());
                    EncodedQuad::new(subject, predicate, object, graph_name)
                }).collect::<Vec<_>>().into_iter()
        } else {
            let g = graph_name.to_bounded_vec();
            <GspoStore<T>>::iter_key_prefix((g, s, ))
                .map(|(p, o)| {
                    let predicate = EncodedTerm::from_bytes(p.as_ref());
                    let object = EncodedTerm::from_bytes(o.as_ref());
                    EncodedQuad::new(subject, predicate, object, graph_name)
                }).collect::<Vec<_>>().into_iter()
        };
        EncodedQuadsIter::new(EncodedQuadIter { iter })
    }

    // Step8 pattern: s
    fn quads_for_subject(
        &self,
        subject: EncodedTerm,
    ) -> EncodedQuadsIter {
        let s = subject.to_bounded_vec();
        let default_graph_iter =
            <DefaultSpoStore<T>>::iter_key_prefix((s.clone(), ))
                .map(|(p, o)| {
                    let predicate = EncodedTerm::from_bytes(p.as_ref());
                    let object = EncodedTerm::from_bytes(o.as_ref());
                    EncodedQuad::new(subject, predicate, object, EncodedTerm::DefaultGraph)
                }).collect::<Vec<_>>().into_iter();

        let named_graph_iter =
            <SpogStore<T>>::iter_key_prefix((s, ))
                .map(|(p, o, g)| {
                    let predicate = EncodedTerm::from_bytes(p.as_ref());
                    let object = EncodedTerm::from_bytes(o.as_ref());
                    let graph_name = EncodedTerm::from_bytes(g.as_ref());
                    EncodedQuad::new(subject, predicate, object, graph_name)
                }).collect::<Vec<_>>().into_iter();

        EncodedQuadsIter::pair(
            EncodedQuadIter { iter: default_graph_iter },
            EncodedQuadIter { iter: named_graph_iter },
        )
    }

    // Step9 pattern: pog
    fn quads_for_predicate_object_graph(
        &self,
        predicate: EncodedTerm,
        object: EncodedTerm,
        graph_name: EncodedTerm,
    ) -> EncodedQuadsIter {
        let p = predicate.to_bounded_vec();
        let o = object.to_bounded_vec();

        let iter = if graph_name.is_default_graph() {
            <DefaultPosStore<T>>::iter_key_prefix((p, o, ))
                .map(|s| {
                    let subject = EncodedTerm::from_bytes(s.as_ref());
                    EncodedQuad::new(subject, predicate, object, graph_name)
                }).collect::<Vec<_>>().into_iter()
        } else {
            let g = graph_name.to_bounded_vec();
            <GposStore<T>>::iter_key_prefix((g, p, o, ))
                .map(|s| {
                    let subject = EncodedTerm::from_bytes(s.as_ref());
                    EncodedQuad::new(subject, predicate, object, graph_name)
                }).collect::<Vec<_>>().into_iter()
        };
        EncodedQuadsIter::new(EncodedQuadIter { iter })
    }

    // Step10 pattern: po
    fn quads_for_predicate_object(
        &self,
        predicate: EncodedTerm,
        object: EncodedTerm,
    ) -> EncodedQuadsIter {
        let p = predicate.to_bounded_vec();
        let o = object.to_bounded_vec();

        let default_graph_iter =
            <DefaultPosStore<T>>::iter_key_prefix((p.clone(), o.clone(), ))
                .map(|s| {
                    let subject = EncodedTerm::from_bytes(s.as_ref());
                    EncodedQuad::new(subject, predicate, object, EncodedTerm::DefaultGraph)
                }).collect::<Vec<_>>().into_iter();

        let named_graph_iter =
            <PosgStore<T>>::iter_key_prefix((p, o, ))
                .map(|(s, g)| {
                    let subject = EncodedTerm::from_bytes(s.as_ref());
                    let graph_name = EncodedTerm::from_bytes(g.as_ref());
                    EncodedQuad::new(subject, predicate, object, graph_name)
                }).collect::<Vec<_>>().into_iter();

        EncodedQuadsIter::pair(
            EncodedQuadIter { iter: default_graph_iter },
            EncodedQuadIter { iter: named_graph_iter },
        )
    }

    // Step11 pattern: pg
    fn quads_for_predicate_graph(
        &self,
        predicate: EncodedTerm,
        graph_name: EncodedTerm,
    ) -> EncodedQuadsIter {
        let p = predicate.to_bounded_vec();

        let iter = if graph_name.is_default_graph() {
            <DefaultPosStore<T>>::iter_key_prefix((p, ))
                .map(|(o, s)| {
                    let object = EncodedTerm::from_bytes(o.as_ref());
                    let subject = EncodedTerm::from_bytes(s.as_ref());
                    EncodedQuad::new(subject, predicate, object, graph_name)
                }).collect::<Vec<_>>().into_iter()
        } else {
            let g = graph_name.to_bounded_vec();
            <GposStore<T>>::iter_key_prefix((g, p, ))
                .map(|(o, s)| {
                    let object = EncodedTerm::from_bytes(o.as_ref());
                    let subject = EncodedTerm::from_bytes(s.as_ref());
                    EncodedQuad::new(subject, predicate, object, graph_name)
                }).collect::<Vec<_>>().into_iter()
        };
        EncodedQuadsIter::new(EncodedQuadIter { iter })
    }

    // Step12 pattern: p
    fn quads_for_predicate(
        &self,
        predicate: EncodedTerm,
    ) -> EncodedQuadsIter {
        let p = predicate.to_bounded_vec();

        let default_graph_iter =
            <DefaultPosStore<T>>::iter_key_prefix((p.clone(), ))
                .map(|(o, s)| {
                    let object = EncodedTerm::from_bytes(o.as_ref());
                    let subject = EncodedTerm::from_bytes(s.as_ref());
                    EncodedQuad::new(subject, predicate, object, EncodedTerm::DefaultGraph)
                }).collect::<Vec<_>>().into_iter();

        let named_graph_iter =
            <PosgStore<T>>::iter_key_prefix((p, ))
                .map(|(o, s, g)| {
                    let object = EncodedTerm::from_bytes(o.as_ref());
                    let subject = EncodedTerm::from_bytes(s.as_ref());
                    let graph_name = EncodedTerm::from_bytes(g.as_ref());
                    EncodedQuad::new(subject, predicate, object, graph_name)
                }).collect::<Vec<_>>().into_iter();

        EncodedQuadsIter::pair(
            EncodedQuadIter { iter: default_graph_iter },
            EncodedQuadIter { iter: named_graph_iter },
        )
    }

    // Step13 pattern: og
    fn quads_for_object_graph(
        &self,
        object: EncodedTerm,
        graph_name: EncodedTerm,
    ) -> EncodedQuadsIter {
        let o = object.to_bounded_vec();

        let iter = if graph_name.is_default_graph() {
            <DefaultOspStore<T>>::iter_key_prefix((o, ))
                .map(|(s, p)| {
                    let subject = EncodedTerm::from_bytes(s.as_ref());
                    let predicate = EncodedTerm::from_bytes(p.as_ref());
                    EncodedQuad::new(subject, predicate, object, graph_name)
                }).collect::<Vec<_>>().into_iter()
        } else {
            let g = graph_name.to_bounded_vec();
            <GospStore<T>>::iter_key_prefix((g, o, ))
                .map(|(s, p)| {
                    let subject = EncodedTerm::from_bytes(s.as_ref());
                    let predicate = EncodedTerm::from_bytes(p.as_ref());
                    EncodedQuad::new(subject, predicate, object, graph_name)
                }).collect::<Vec<_>>().into_iter()
        };
        EncodedQuadsIter::new(EncodedQuadIter { iter })
    }

    // Step14 pattern: o
    fn quads_for_object(
        &self,
        object: EncodedTerm,
    ) -> EncodedQuadsIter {
        let o = object.to_bounded_vec();

        let default_graph_iter =
            <DefaultOspStore<T>>::iter_key_prefix((o.clone(), ))
                .map(|(s, p)| {
                    let subject = EncodedTerm::from_bytes(s.as_ref());
                    let predicate = EncodedTerm::from_bytes(p.as_ref());
                    EncodedQuad::new(subject, predicate, object, EncodedTerm::DefaultGraph)
                }).collect::<Vec<_>>().into_iter();

        let named_graph_iter =
            <OspgStore<T>>::iter_key_prefix((o, ))
                .map(|(s, p, g)| {
                    let subject = EncodedTerm::from_bytes(s.as_ref());
                    let predicate = EncodedTerm::from_bytes(p.as_ref());
                    let graph_name = EncodedTerm::from_bytes(g.as_ref());
                    EncodedQuad::new(subject, predicate, object, graph_name)
                }).collect::<Vec<_>>().into_iter();

        EncodedQuadsIter::pair(
            EncodedQuadIter { iter: default_graph_iter },
            EncodedQuadIter { iter: named_graph_iter },
        )
    }

    // Step15 pattern: g
    fn quads_for_graph(&self, graph_name: EncodedTerm) -> EncodedQuadsIter {
        let iter = if graph_name.is_default_graph() {
            <DefaultSpoStore<T>>::iter_keys()
                .map(|(s, p, o)| {
                    let subject = EncodedTerm::from_bytes(s.as_ref());
                    let predicate = EncodedTerm::from_bytes(p.as_ref());
                    let object = EncodedTerm::from_bytes(o.as_ref());
                    EncodedQuad::new(subject, predicate, object, graph_name)
                }).collect::<Vec<_>>().into_iter()
        } else {
            let g = graph_name.to_bounded_vec();
            <GspoStore<T>>::iter_key_prefix((g, ))
                .map(|(s, p, o)| {
                    let subject = EncodedTerm::from_bytes(s.as_ref());
                    let predicate = EncodedTerm::from_bytes(p.as_ref());
                    let object = EncodedTerm::from_bytes(o.as_ref());
                    EncodedQuad::new(subject, predicate, object, graph_name)
                }).collect::<Vec<_>>().into_iter()
        };
        EncodedQuadsIter::new(EncodedQuadIter { iter })
    }

    // Step16 pattern: none (query all)
    fn quads(&self) -> EncodedQuadsIter {
        let default_graph_iter =
            <DefaultSpoStore<T>>::iter_keys()
                .map(|(s, p, o)| {
                    let subject = EncodedTerm::from_bytes(s.as_ref());
                    let predicate = EncodedTerm::from_bytes(p.as_ref());
                    let object = EncodedTerm::from_bytes(o.as_ref());
                    EncodedQuad::new(subject, predicate, object, EncodedTerm::DefaultGraph)
                }).collect::<Vec<_>>().into_iter();

        let named_graph_iter =
            <GspoStore<T>>::iter_keys()
                .map(|(g, s, p, o)| {
                    let graph_name = EncodedTerm::from_bytes(g.as_ref());
                    let subject = EncodedTerm::from_bytes(s.as_ref());
                    let predicate = EncodedTerm::from_bytes(p.as_ref());
                    let object = EncodedTerm::from_bytes(o.as_ref());
                    EncodedQuad::new(subject, predicate, object, graph_name)
                }).collect::<Vec<_>>().into_iter();

        EncodedQuadsIter::pair(
            EncodedQuadIter { iter: default_graph_iter },
            EncodedQuadIter { iter: named_graph_iter },
        )
    }
}

#[derive(Debug)]
enum StoreFamily {
    DefaultSpo,
    //TODO rename to Dspo , same as sled impl, or my prefer: dspo(default), spog (named graph)
    DefaultPos,
    DefaultOsp,
    Gspo,
    Gpos,
    Gosp,
    Spog,
    Posg,
    Ospg,
}

//****************************
// ID/STR Mapping Store
//****************************

impl<T> StrEncodingAware for GraphStore<T> {
    type Error = Infallible;
    type StrId = StrHash;
}

impl<T: Config> StrLookup for GraphStore<T> {
    fn get_str(&self, id: StrHash) -> Result<Option<String>, Infallible> {
        Ok(if let Some(value) = <Id2StrStore<T>>::get(*id) {
            Some(String::from_utf8(value.into_inner()).unwrap()) //TODO check unwrap
        } else {
            None
        })
    }

    fn get_str_id(&self, value: &str) -> Result<Option<StrHash>, Infallible> {
        let id = StrHash::new(value);
        Ok(if <Id2StrStore<T>>::contains_key(*id) {
            Some(id)
        } else {
            None
        })
    }
}

impl<'a, T: Config> StrContainer for &'a GraphStore<T> {
    fn insert_str(&mut self, value: &str) -> Result<StrHash, Infallible> {
        let key = StrHash::new(value);
        let value = BoundedVec::try_from(value.as_bytes().to_vec()).unwrap(); //TODO check unwrap
        <Id2StrStore<T>>::insert(*key, value);
        Ok(key)
    }
}

//****************************
// READ Store
//****************************
impl<'a, T: Config> ReadableEncodedStore for GraphStore<T> {
    type QuadsIter = EncodedQuadsIter;
    type GraphsIter = EncodedGraphNameIter;

    fn encoded_quads_for_pattern(
        &self,
        subject: Option<EncodedTerm>,
        predicate: Option<EncodedTerm>,
        object: Option<EncodedTerm>,
        graph_name: Option<EncodedTerm>,
    ) -> EncodedQuadsIter {
        self.encoded_quads_for_pattern(subject, predicate, object, graph_name)
    }

    fn encoded_named_graphs(&self) -> Self::GraphsIter {
        let iter = <GraphNameStore<T>>::iter_keys()
            .map(|graph_name| {
                EncodedTerm::from_bytes(graph_name.as_ref())
            }).collect::<Vec<_>>().into_iter();
        EncodedGraphNameIter { iter }
    }

    fn contains_encoded_named_graph(&self, graph_name: EncodedTerm) -> Result<bool, Infallible> {
        let g = graph_name.to_bounded_vec();
        Ok(<GraphNameStore<T>>::contains_key(g))
    }
}

//****************************
// Store iterator
//****************************
struct EncodedQuadsIter {
    first: EncodedQuadIter,
    second: Option<EncodedQuadIter>,
}

impl EncodedQuadsIter {
    fn new(first: EncodedQuadIter) -> Self {
        Self {
            first,
            second: None,
        }
    }

    fn pair(first: EncodedQuadIter, second: EncodedQuadIter) -> Self {
        Self {
            first,
            second: Some(second),
        }
    }
}

impl Iterator for EncodedQuadsIter {
    type Item = Result<EncodedQuad, Infallible>;

    fn next(&mut self) -> Option<Result<EncodedQuad, Infallible>> {
        if let Some(result) = self.first.next() {
            Some(result)
        } else if let Some(second) = self.second.as_mut() {
            second.next()
        } else {
            None
        }
    }
}

struct EncodedQuadIter {
    iter: IntoIter<EncodedQuad>,
}

impl Iterator for EncodedQuadIter {
    type Item = Result<EncodedQuad, Infallible>;

    fn next(&mut self) -> Option<Result<EncodedQuad, Infallible>> {
        self.iter.next().map(Ok)
    }
}

struct EncodedGraphNameIter {
    iter: IntoIter<EncodedTerm>,
}

impl Iterator for EncodedGraphNameIter {
    type Item = Result<EncodedTerm, Infallible>;

    fn next(&mut self) -> Option<Result<EncodedTerm, Infallible>> {
        self.iter.next().map(Ok)
    }
}

/// An iterator returning the graph names.
pub struct GraphNameIter<T: Config> {
    iter: EncodedGraphNameIter,
    store: GraphStore<T>,
}

impl<T: Config> Iterator for GraphNameIter<T> {
    type Item = NamedOrBlankNode;

    fn next(&mut self) -> Option<NamedOrBlankNode> {
        if let Some(Ok(encoded_term)) = self.iter.next() {
            Some(self.store.decode_named_or_blank_node(encoded_term).unwrap())
        } else {
            None
        }
    }

    fn size_hint(&self) -> (usize, Option<usize>) {
        self.iter.size_hint()
    }
}

/// An iterator returning the quads
pub struct QuadIter<T: Config> {
    inner: QuadIterInner<T>,
}

impl<T: Config> QuadIter<T> {
    fn new(iter: EncodedQuadsIter, store: GraphStore<T>) -> Self {
        Self {
            inner: QuadIterInner::Quads {
                iter,
                store,
            }
        }
    }

    fn empty() -> Self {
        Self { inner: QuadIterInner::Empty }
    }
}

enum QuadIterInner<T: Config> {
    Quads {
        iter: EncodedQuadsIter,
        store: GraphStore<T>,
    },
    Empty,
}

impl<T: Config> Iterator for QuadIter<T> {
    type Item = Quad;

    fn next(&mut self) -> Option<Quad> {
        match &mut self.inner {
            QuadIterInner::Quads { iter, store } => {
                if let Some(Ok(q)) = iter.next() {
                    let quad = store.decode_quad(&q).unwrap();
                    Some(quad)
                } else {
                    None
                }
            }
            QuadIterInner::Empty => None,
        }
    }
}

//****************************
// WRITE Store
//****************************
impl EncodedTerm {
    fn to_bounded_vec<S: Get<u32>>(self) -> BoundedVec<u8, S> {
        BoundedVec::try_from(self.to_bytes()).unwrap()
    }
}

fn insert_into_triple_map<T: Config>(store_family: StoreFamily, t: (EncodedTerm, EncodedTerm, EncodedTerm)) {
    let triple_key = (t.0.to_bounded_vec(), t.1.to_bounded_vec(), t.2.to_bounded_vec());
    match store_family {
        StoreFamily::DefaultSpo => <DefaultSpoStore<T>>::insert(triple_key, true),
        StoreFamily::DefaultPos => <DefaultPosStore<T>>::insert(triple_key, true),
        StoreFamily::DefaultOsp => <DefaultOspStore<T>>::insert(triple_key, true),
        _ => panic!("Unsupported triple StoreFamily: {:?}", store_family),
    }
}

fn remove_from_triple_map<T: Config>(store_family: StoreFamily, t: (EncodedTerm, EncodedTerm, EncodedTerm)) {
    let triple_key = (t.0.to_bounded_vec(), t.1.to_bounded_vec(), t.2.to_bounded_vec());
    match store_family {
        StoreFamily::DefaultSpo => <DefaultSpoStore<T>>::remove(triple_key),
        StoreFamily::DefaultPos => <DefaultPosStore<T>>::remove(triple_key),
        StoreFamily::DefaultOsp => <DefaultOspStore<T>>::remove(triple_key),
        _ => panic!("Unsupported triple StoreFamily: {:?}", store_family),
    }
}

fn insert_into_quad_map<T: Config>(store_family: StoreFamily, t: (EncodedTerm, EncodedTerm, EncodedTerm, EncodedTerm)) {
    let quad_key = (t.0.to_bounded_vec(), t.1.to_bounded_vec(), t.2.to_bounded_vec(), t.3.to_bounded_vec());
    match store_family {
        StoreFamily::Gspo => <GspoStore<T>>::insert(quad_key, true),
        StoreFamily::Gpos => <GposStore<T>>::insert(quad_key, true),
        StoreFamily::Gosp => <GospStore<T>>::insert(quad_key, true),
        StoreFamily::Spog => <SpogStore<T>>::insert(quad_key, true),
        StoreFamily::Posg => <PosgStore<T>>::insert(quad_key, true),
        StoreFamily::Ospg => <OspgStore<T>>::insert(quad_key, true),
        _ => panic!("Unsupported quad StoreFamily: {:?}", store_family),
    }
}

fn remove_from_quad_map<T: Config>(store_family: StoreFamily, t: (EncodedTerm, EncodedTerm, EncodedTerm, EncodedTerm)) {
    let quad_key = (t.0.to_bounded_vec(), t.1.to_bounded_vec(), t.2.to_bounded_vec(), t.3.to_bounded_vec());
    match store_family {
        StoreFamily::Gspo => <GspoStore<T>>::remove(quad_key),
        StoreFamily::Gpos => <GposStore<T>>::remove(quad_key),
        StoreFamily::Gosp => <GospStore<T>>::remove(quad_key),
        StoreFamily::Spog => <SpogStore<T>>::remove(quad_key),
        StoreFamily::Posg => <PosgStore<T>>::remove(quad_key),
        StoreFamily::Ospg => <OspgStore<T>>::remove(quad_key),
        _ => panic!("Unsupported quad StoreFamily: {:?}", store_family),
    }
}

impl<'a, T: Config> WritableEncodedStore for &'a GraphStore<T> {
    fn insert_encoded(&mut self, quad: &EncodedQuad) -> Result<(), Infallible> {
        if quad.graph_name.is_default_graph() {
            insert_into_triple_map::<T>(
                StoreFamily::DefaultSpo,
                (quad.subject, quad.predicate, quad.object),
            );
            insert_into_triple_map::<T>(
                StoreFamily::DefaultPos,
                (quad.predicate, quad.object, quad.subject),
            );
            insert_into_triple_map::<T>(
                StoreFamily::DefaultOsp,
                (quad.object, quad.subject, quad.predicate),
            );
        } else {
            insert_into_quad_map::<T>(
                StoreFamily::Gspo,
                (quad.graph_name, quad.subject, quad.predicate, quad.object),
            );
            insert_into_quad_map::<T>(
                StoreFamily::Gpos,
                (quad.graph_name, quad.predicate, quad.object, quad.subject),
            );
            insert_into_quad_map::<T>(
                StoreFamily::Gosp,
                (quad.graph_name, quad.object, quad.subject, quad.predicate),
            );
            insert_into_quad_map::<T>(
                StoreFamily::Spog,
                (quad.subject, quad.predicate, quad.object, quad.graph_name),
            );
            insert_into_quad_map::<T>(
                StoreFamily::Posg,
                (quad.predicate, quad.object, quad.subject, quad.graph_name),
            );
            insert_into_quad_map::<T>(
                StoreFamily::Ospg,
                (quad.object, quad.subject, quad.predicate, quad.graph_name),
            );

            // store graph name
            <GraphNameStore<T>>::insert(quad.graph_name.to_bounded_vec(), true);
        }
        Ok(())
    }

    fn remove_encoded(&mut self, quad: &EncodedQuad) -> Result<(), Infallible> {
        if quad.graph_name.is_default_graph() {
            remove_from_triple_map::<T>(
                StoreFamily::DefaultSpo,
                (quad.subject, quad.predicate, quad.object),
            );
            remove_from_triple_map::<T>(
                StoreFamily::DefaultPos,
                (quad.predicate, quad.object, quad.subject),
            );
            remove_from_triple_map::<T>(
                StoreFamily::DefaultOsp,
                (quad.object, quad.subject, quad.predicate),
            );
        } else {
            remove_from_quad_map::<T>(
                StoreFamily::Gspo,
                (quad.graph_name, quad.subject, quad.predicate, quad.object),
            );
            remove_from_quad_map::<T>(
                StoreFamily::Gpos,
                (quad.graph_name, quad.predicate, quad.object, quad.subject),
            );
            remove_from_quad_map::<T>(
                StoreFamily::Gosp,
                (quad.graph_name, quad.object, quad.subject, quad.predicate),
            );
            remove_from_quad_map::<T>(
                StoreFamily::Spog,
                (quad.subject, quad.predicate, quad.object, quad.graph_name),
            );
            remove_from_quad_map::<T>(
                StoreFamily::Posg,
                (quad.predicate, quad.object, quad.subject, quad.graph_name),
            );
            remove_from_quad_map::<T>(
                StoreFamily::Ospg,
                (quad.object, quad.subject, quad.predicate, quad.graph_name),
            );
        }
        Ok(())
    }

    fn insert_encoded_named_graph(&mut self, graph_name: EncodedTerm) -> Result<(), Infallible> {
        <GraphNameStore<T>>::insert(graph_name.to_bounded_vec(), true);
        Ok(())
    }

    fn clear_encoded_graph(&mut self, graph_name: EncodedTerm) -> Result<(), Infallible> {
        if graph_name.is_default_graph() {
            <DefaultSpoStore<T>>::remove_all(None);
            <DefaultPosStore<T>>::remove_all(None);
            <DefaultOspStore<T>>::remove_all(None);
        } else {
            for quad in self.quads_for_graph(graph_name) {
                self.remove_encoded(&quad?)?;
            }
        }
        Ok(())
    }

    fn remove_encoded_named_graph(&mut self, graph_name: EncodedTerm) -> Result<(), Infallible> {
        for quad in self.quads_for_graph(graph_name) {
            self.remove_encoded(&quad?)?;
        }
        // remove graph name
        <GraphNameStore<T>>::remove(graph_name.to_bounded_vec());
        Ok(())
    }

    fn clear(&mut self) -> Result<(), Self::Error> {
        // clear graph names
        <GraphNameStore<T>>::remove_all(None);
        // clear id/string
        <Id2StrStore<T>>::remove_all(None);
        // clear default graph
        <DefaultSpoStore<T>>::remove_all(None);
        <DefaultPosStore<T>>::remove_all(None);
        <DefaultOspStore<T>>::remove_all(None);
        // clear named graph
        <GspoStore<T>>::remove_all(None);
        <GposStore<T>>::remove_all(None);
        <GospStore<T>>::remove_all(None);
        <SpogStore<T>>::remove_all(None);
        <PosgStore<T>>::remove_all(None);
        <OspgStore<T>>::remove_all(None);
        Ok(())
    }
}