//! Deployment planning and tracking for Tempo environments.

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use tempoforge_blockchain::Network;
use tempoforge_common::{DeploymentId, ProjectId};
use uuid::Uuid;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum DeploymentStatus {
    Pending,
    Submitted,
    Confirmed,
    Verified,
    Failed,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DeploymentRecord {
    pub id: DeploymentId,
    pub project_id: ProjectId,
    pub network: Network,
    pub contract_name: String,
    pub address: Option<String>,
    pub tx_hash: Option<String>,
    pub status: DeploymentStatus,
    pub artifact: serde_json::Value,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DeploymentPlan {
    pub network: Network,
    pub rpc_url: String,
    pub chain_id: u64,
    pub steps: Vec<String>,
    pub foundry_script: String,
}

pub struct DeploymentService;

impl DeploymentService {
    pub fn plan(network: Network, contract_name: &str) -> DeploymentPlan {
        let rpc = network.rpc_url();
        let chain_id = network.chain_id();
        let foundry_script = format!(
            r#"// SPDX-License-Identifier: MIT
pragma solidity ^0.8.24;

import "forge-std/Script.sol";
import "../src/{contract_name}.sol";

contract Deploy{contract_name} is Script {{
    function run() external {{
        uint256 pk = vm.envUint("PRIVATE_KEY");
        vm.startBroadcast(pk);
        {contract_name} deployed = new {contract_name}();
        console2.log("deployed", address(deployed));
        vm.stopBroadcast();
    }}
}}
"#
        );

        DeploymentPlan {
            network,
            rpc_url: rpc.clone(),
            chain_id,
            steps: vec![
                format!("export TEMPO_RPC_URL={rpc}"),
                format!("forge script script/Deploy{contract_name}.s.sol --rpc-url $TEMPO_RPC_URL --broadcast"),
                "Record tx hash and contract address in TempoForge".into(),
                "Verify source once explorer verification API is available".into(),
            ],
            foundry_script,
        }
    }

    pub fn create_pending(
        project_id: ProjectId,
        network: Network,
        contract_name: impl Into<String>,
    ) -> DeploymentRecord {
        let now = Utc::now();
        DeploymentRecord {
            id: DeploymentId(Uuid::new_v4()),
            project_id,
            network,
            contract_name: contract_name.into(),
            address: None,
            tx_hash: None,
            status: DeploymentStatus::Pending,
            artifact: serde_json::json!({}),
            created_at: now,
            updated_at: now,
        }
    }
}
