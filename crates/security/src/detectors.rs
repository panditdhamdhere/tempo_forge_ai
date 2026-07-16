use regex::Regex;
use tempoforge_ai_engine::types::{Finding, Severity};

pub fn analyze_source(source: &str) -> Vec<Finding> {
    let mut findings = Vec::new();

    if contains_call_value_before_state(source) {
        findings.push(Finding {
            severity: Severity::High,
            title: "Possible reentrancy via call{value}".into(),
            description: "External call with value appears before state finalization patterns. \
                 Review checks-effects-interactions."
                .into(),
            location: find_line(source, ".call{"),
            recommendation: "Update state before external calls; use ReentrancyGuard.".into(),
            diff: Some(
                "- (bool ok,) = target.call{value: amount}(\"\");\n+ // effects first, then interaction\n"
                    .into(),
            ),
        });
    }

    if Regex::new(r"\btx\.origin\b").unwrap().is_match(source) {
        findings.push(Finding {
            severity: Severity::High,
            title: "Authorization via tx.origin".into(),
            description: "tx.origin is phishing-prone and should not authorize privileged actions."
                .into(),
            location: find_line(source, "tx.origin"),
            recommendation: "Use msg.sender with Ownable/AccessControl.".into(),
            diff: Some("- require(tx.origin == owner);\n+ require(msg.sender == owner);\n".into()),
        });
    }

    if Regex::new(r"\.delegatecall\s*\(").unwrap().is_match(source)
        && !source.contains("onlyOwner")
        && !source.contains("onlyRole")
    {
        findings.push(Finding {
            severity: Severity::Critical,
            title: "Unchecked delegatecall".into(),
            description: "delegatecall found without clear access control markers.".into(),
            location: find_line(source, "delegatecall"),
            recommendation: "Restrict delegatecall targets; validate implementation addresses."
                .into(),
            diff: None,
        });
    }

    if Regex::new(r"selfdestruct\s*\(").unwrap().is_match(source) {
        findings.push(Finding {
            severity: Severity::Medium,
            title: "selfdestruct present".into(),
            description: "selfdestruct can brick integrations and is restricted in newer EVM rules."
                .into(),
            location: find_line(source, "selfdestruct"),
            recommendation: "Prefer pause + migrate patterns over selfdestruct.".into(),
            diff: None,
        });
    }

    if source.contains("block.timestamp")
        && (source.contains("random") || source.contains("Random"))
    {
        findings.push(Finding {
            severity: Severity::Medium,
            title: "Weak randomness from block.timestamp".into(),
            description: "block.timestamp is miner/validator influenced and unsuitable for randomness."
                .into(),
            location: find_line(source, "block.timestamp"),
            recommendation: "Use a VRF or commit-reveal scheme.".into(),
            diff: None,
        });
    }

    if Regex::new(r"transfer\s*\(\s*[^)]+\s*\)").unwrap().is_match(source)
        && source.contains("for (")
    {
        findings.push(Finding {
            severity: Severity::Medium,
            title: "Push payment in loop".into(),
            description: "Sending ETH/tokens in a loop can DOS the function if one recipient reverts."
                .into(),
            location: find_line(source, "transfer("),
            recommendation: "Use pull-over-push withdrawal patterns.".into(),
            diff: None,
        });
    }

    if !source.contains("nonReentrant")
        && (source.contains("withdraw") || source.contains("Withdraw"))
        && source.contains(".call")
    {
        findings.push(Finding {
            severity: Severity::Medium,
            title: "Withdraw path without ReentrancyGuard".into(),
            description: "Withdraw-like function uses low-level call without nonReentrant.".into(),
            location: find_line(source, "withdraw"),
            recommendation: "Add OpenZeppelin ReentrancyGuard.".into(),
            diff: Some(
                "+ import {ReentrancyGuard} from \"@openzeppelin/contracts/utils/ReentrancyGuard.sol\";\n"
                    .into(),
            ),
        });
    }

    if (source.contains("upgradeTo") || source.contains("UUPSUpgradeable"))
        && !source.contains("_authorizeUpgrade")
        && !source.contains("onlyOwner")
    {
        findings.push(Finding {
            severity: Severity::High,
            title: "Upgradeability risk".into(),
            description: "Upgradeable pattern detected without clear upgrade authorization."
                .into(),
            location: find_line(source, "upgrade"),
            recommendation: "Implement _authorizeUpgrade with strict access control.".into(),
            diff: None,
        });
    }

    findings
}

fn contains_call_value_before_state(source: &str) -> bool {
    let call_pos = source.find(".call{");
    let state_markers = ["balances[", "totalSupply", "mapping"];
    match call_pos {
        Some(pos) => {
            let after = &source[pos..];
            state_markers.iter().any(|m| after.contains(m))
                || (!source[..pos].contains("balances[") && source.contains("balances["))
        }
        None => false,
    }
}

fn find_line(source: &str, needle: &str) -> Option<String> {
    source
        .lines()
        .enumerate()
        .find(|(_, line)| line.contains(needle))
        .map(|(idx, line)| format!("L{}: {}", idx + 1, line.trim()))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn detects_tx_origin() {
        let src = "function admin() external { require(tx.origin == owner); }";
        let findings = analyze_source(src);
        assert!(findings.iter().any(|f| f.title.contains("tx.origin")));
    }
}
