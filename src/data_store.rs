use async_trait::async_trait;
use nodespace_core_types::{Node, NodeId, NodeSpaceResult};

// DataStore trait - authoritative interface owned by this repository
#[async_trait]
pub trait DataStore {
    async fn store_node(&self, node: Node) -> NodeSpaceResult<NodeId>;
    async fn get_node(&self, id: &NodeId) -> NodeSpaceResult<Option<Node>>;
    async fn update_node(&self, node: Node) -> NodeSpaceResult<()>;
    async fn update_node_with_embedding(
        &self,
        node: Node,
        embedding: Vec<f32>,
    ) -> NodeSpaceResult<()>;
    async fn delete_node(&self, id: &NodeId) -> NodeSpaceResult<()>;
    async fn query_nodes(&self, query: &str) -> NodeSpaceResult<Vec<Node>>;
    async fn create_relationship(
        &self,
        from: &NodeId,
        to: &NodeId,
        rel_type: &str,
    ) -> NodeSpaceResult<()>;

    // Vector search capabilities
    async fn store_node_with_embedding(
        &self,
        node: Node,
        embedding: Vec<f32>,
    ) -> NodeSpaceResult<NodeId>;
    async fn search_similar_nodes(
        &self,
        embedding: Vec<f32>,
        limit: usize,
    ) -> NodeSpaceResult<Vec<(Node, f32)>>;
    async fn update_node_embedding(&self, id: &NodeId, embedding: Vec<f32>) -> NodeSpaceResult<()>;

    // Semantic search with provided embedding vector
    async fn semantic_search_with_embedding(
        &self,
        embedding: Vec<f32>,
        limit: usize,
    ) -> NodeSpaceResult<Vec<(Node, f32)>>;

    // NEW: Multi-level embedding methods for NS-94
    async fn store_node_with_multi_embeddings(
        &self,
        node: Node,
        embeddings: MultiLevelEmbeddings,
    ) -> NodeSpaceResult<NodeId>;
    async fn update_node_embeddings(
        &self,
        node_id: &NodeId,
        embeddings: MultiLevelEmbeddings,
    ) -> NodeSpaceResult<()>;
    async fn get_node_embeddings(
        &self,
        node_id: &NodeId,
    ) -> NodeSpaceResult<Option<MultiLevelEmbeddings>>;

    // NEW: Embedding-specific search methods
    async fn search_by_individual_embedding(
        &self,
        embedding: Vec<f32>,
        limit: usize,
    ) -> NodeSpaceResult<Vec<(Node, f32)>>;
    async fn search_by_contextual_embedding(
        &self,
        embedding: Vec<f32>,
        limit: usize,
    ) -> NodeSpaceResult<Vec<(Node, f32)>>;
    async fn search_by_hierarchical_embedding(
        &self,
        embedding: Vec<f32>,
        limit: usize,
    ) -> NodeSpaceResult<Vec<(Node, f32)>>;

    // NEW: Hybrid search combining multiple levels
    async fn hybrid_semantic_search(
        &self,
        embeddings: QueryEmbeddings,
        config: HybridSearchConfig,
    ) -> NodeSpaceResult<Vec<SearchResult>>;

    // Existing cross-modal search methods for NS-81
    async fn create_image_node(&self, image_node: ImageNode) -> NodeSpaceResult<String>;
    async fn get_image_node(&self, id: &str) -> NodeSpaceResult<Option<ImageNode>>;
    async fn search_multimodal(
        &self,
        query_embedding: Vec<f32>,
        types: Vec<NodeType>,
    ) -> NodeSpaceResult<Vec<Node>>;
    async fn hybrid_multimodal_search(
        &self,
        query_embedding: Vec<f32>,
        config: &HybridSearchConfig,
    ) -> NodeSpaceResult<Vec<SearchResult>>;
}

// Cross-modal types for NS-81 implementation
#[derive(Debug, Clone)]
pub struct ImageNode {
    pub id: String,
    pub image_data: Vec<u8>, // Raw image bytes
    pub embedding: Vec<f32>, // CLIP vision embedding (512-dim)
    pub metadata: ImageMetadata,
    pub created_at: chrono::DateTime<chrono::Utc>,
}

#[derive(Debug, Clone)]
pub struct ImageMetadata {
    pub filename: String,
    pub mime_type: String,
    pub width: u32,
    pub height: u32,
    pub exif_data: Option<serde_json::Value>,
    pub description: Option<String>,
}

#[derive(Debug, Clone)]
pub enum NodeType {
    Text,
    Image,
    Date,
    Task,
}

#[derive(Debug, Clone)]
pub struct HybridSearchConfig {
    pub semantic_weight: f64,            // 0.0-1.0, semantic similarity
    pub structural_weight: f64,          // 0.0-1.0, relationship proximity
    pub temporal_weight: f64,            // 0.0-1.0, time-based relevance
    pub individual_weight: f64,          // 0.0-1.0, individual embedding weight
    pub contextual_weight: f64,          // 0.0-1.0, contextual embedding weight
    pub hierarchical_weight: f64,        // 0.0-1.0, hierarchical embedding weight
    pub max_results: usize,              // Maximum results to return
    pub min_similarity_threshold: f64,   // Minimum similarity score
    pub enable_cross_modal: bool,        // Allow text→image search
    pub enable_cross_level_fusion: bool, // Combine scores across embedding levels
    pub search_timeout_ms: u64,          // Maximum search time
}

#[derive(Debug, Clone)]
pub struct SearchResult {
    pub node: Node,
    pub score: f32,
    pub relevance_factors: RelevanceFactors,
}

#[derive(Debug, Clone)]
pub struct RelevanceFactors {
    pub semantic_score: f32,
    pub structural_score: f32,
    pub temporal_score: f32,
    pub cross_modal_score: Option<f32>,
}

// NEW: Multi-level embedding types for NS-94
#[derive(Debug, Clone)]
pub struct MultiLevelEmbeddings {
    pub individual: Vec<f32>,            // Node content embedding
    pub contextual: Option<Vec<f32>>,    // Context-aware embedding (with siblings/parent)
    pub hierarchical: Option<Vec<f32>>,  // Hierarchical path embedding
    pub embedding_model: Option<String>, // Model used for generation
    pub generated_at: chrono::DateTime<chrono::Utc>,
}

#[derive(Debug, Clone)]
pub struct QueryEmbeddings {
    pub individual: Vec<f32>,
    pub contextual: Option<Vec<f32>>,
    pub hierarchical: Option<Vec<f32>>,
}
