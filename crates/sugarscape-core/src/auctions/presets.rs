//! Source-labelled experiments, all with explicit reconstruction conventions.
use super::config::*;
use crate::{model::ModelConfig, presets::ModelPreset};
pub fn presets() -> Vec<ModelPreset> {
    let mut out = vec![];
    {
        let c = AuctionsConfig::default();
        out.push(ModelPreset {
            id: "auctions-first-price",
            name: "Two bidders learn how much to pay",
            source: "Banchio & Skrzypacz 2022, arXiv 2202.05947v1",
            description: "Starting Q=1/(1-discount), approximately 100 at .99; sampled auction ties, expected hindsight ties. Numerical initialization and tie conventions are our reconstruction. Fixed horizon, final stability selection. Figure 1a: 1,000 terminal-policy sessions; plotted mean .2265, prose .24. Native registered mean .232500 (1,000/1,000 stable); figure and prose checks hold.",
            config: ModelConfig::Auctions(c),
        });
    }
    {
        let c = AuctionsConfig {
            auction: Auction::SecondPrice,
            ..Default::default()
        };
        out.push(ModelPreset {
            id: "auctions-second-price",
            name: "The winner pays the other bidder’s bid",
            source: "Banchio & Skrzypacz 2022, arXiv 2202.05947v1",
            description: "Starting Q=1/(1-discount), approximately 100 at .99; sampled auction ties, expected hindsight ties. Numerical initialization and tie conventions are our reconstruction. Fixed horizon, final stability selection. Figure 1b: 1,000 terminal-policy sessions; plotted mean .9471, prose .95. Native registered mean .945295 (999/1,000 stable); figure and prose checks hold.",
            config: ModelConfig::Auctions(c),
        });
    }
    {
        let c = AuctionsConfig {
            feedback: Feedback::RivalBids,
            update: Update::All,
            ..Default::default()
        };
        out.push(ModelPreset {
            id: "auctions-feedback",
            name: "Bidders learn from every bid they could have made",
            source: "Banchio & Skrzypacz 2022, arXiv 2202.05947v1",
            description: "Starting Q=1/(1-discount), approximately 100 at .99; sampled auction ties, expected hindsight ties. Numerical initialization and tie conventions are our reconstruction. Fixed horizon, final stability selection. Figure 5a: 500 terminal-policy sessions at (.90,.90). Native registered mean .906800 (500/500 stable); 86.4% at (.90,.90) misses the fixed 90% figure-fit rule.",
            config: ModelConfig::Auctions(c),
        });
    }
    {
        let c = AuctionsConfig {
            feedback: Feedback::RivalBids,
            ..Default::default()
        };
        out.push(ModelPreset {
            id: "auctions-unused-feedback",
            name: "Bidders receive information they do not use",
            source: "Banchio & Skrzypacz 2022, arXiv 2202.05947v1",
            description: "Starting Q=1/(1-discount), approximately 100 at .99; sampled auction ties, expected hindsight ties. Numerical initialization and tie conventions are our reconstruction. Fixed horizon, final stability selection. Project information-unused control for Figure 5; terminal-policy sessions. Native control: identical economic trajectories in all 100 full-horizon paired seeds.",
            config: ModelConfig::Auctions(c),
        });
    }
    {
        let c = AuctionsConfig {
            exploration_set: ExplorationSet::Neighbors,
            ..Default::default()
        };
        out.push(ModelPreset {
            id: "auctions-local",
            name: "Bidders try the next bid up or down",
            source: "Banchio & Skrzypacz 2022, arXiv 2202.05947v1",
            description: "Starting Q=1/(1-discount), approximately 100 at .99; sampled auction ties, expected hindsight ties. Numerical initialization and tie conventions are our reconstruction. Fixed horizon, final stability selection. Result 3/Figure 10: 100 terminal-policy sessions per format; boundary rule reconstructed as available neighbors. Native registered FPA mean .221500 and SPA .950000; all 100 sessions per format stable, directional check holds.",
            config: ModelConfig::Auctions(c),
        });
    }
    {
        let c = AuctionsConfig {
            q_init: QInit::Biased,
            epsilon: 0.25,
            ..Default::default()
        };
        out.push(ModelPreset {
            id: "auctions-biased",
            name: "Bidders start with a preference for low bids",
            source: "Banchio & Skrzypacz 2022, arXiv 2202.05947v1",
            description: "Starting Q(.40)=30, others 0; epsilon=.25. The unpublished Q vector is reconstructed. Sampled auction ties, expected hindsight ties. Fixed horizon, final stability selection. Figure 3 supplies one example trajectory; terminal-policy ensembles here score our specified bias reading. Native named-reading FPA mean .259950 and SPA .948447; directional check holds, unpublished source Q vector remains unresolved.",
            config: ModelConfig::Auctions(c),
        });
    }
    {
        let c = AuctionsConfig {
            downward_trigger: DownwardTrigger::Stable,
            ..Default::default()
        };
        out.push(ModelPreset {
            id: "auctions-downward",
            name: "Bidders are nudged toward lower bids",
            source: "Banchio & Skrzypacz 2022, arXiv 2202.05947v1",
            description: "Starting Q=1/(1-discount), approximately 100 at .99; sampled auction ties, expected hindsight ties. Numerical initialization and tie conventions are our reconstruction. Fixed horizon, final stability selection. Result 4/Figure 4: terminal-policy distribution; stable-window trigger and activation clock are our reading. Native primary reading fails: only 1.50% of stable SPA profiles below top versus required 50%; alternative clocks/triggers retained.",
            config: ModelConfig::Auctions(c),
        });
    }
    {
        let c = AuctionsConfig {
            out_bids: 7,
            ..Default::default()
        };
        out.push(ModelPreset {
            id: "auctions-nonparticipation",
            name: "Bidders can choose to sit out",
            source: "Banchio & Skrzypacz 2022, arXiv 2202.05947v1",
            description: "Starting Q=1/(1-discount), approximately 100 at .99; sampled auction ties, expected hindsight ties. Numerical initialization and tie conventions are our reconstruction. Fixed horizon, final stability selection. Result 6/Figure 6: terminal-policy distribution; added -.30 through 0 mesh is our reading. Native registered FPA mean .106200, .126300 below baseline; extension check holds.",
            config: ModelConfig::Auctions(c),
        });
    }
    {
        let c = AuctionsConfig {
            reserve: 0.2,
            ..Default::default()
        };
        out.push(ModelPreset {
            id: "auctions-reserve",
            name: "The seller sets a minimum bid",
            source: "Banchio & Skrzypacz 2022, arXiv 2202.05947v1",
            description: "Starting Q=1/(1-discount), approximately 100 at .99; sampled auction ties, expected hindsight ties. Numerical initialization and tie conventions are our reconstruction. Fixed horizon, final stability selection. Result 7/Figure 7: terminal-policy distribution; eligibility and reserve payment floor are explicit readings. Native registered mean greedy bid .265150, .032650 above baseline; extension check holds.",
            config: ModelConfig::Auctions(c),
        });
    }
    {
        let c = AuctionsConfig {
            bidders: 3,
            ..Default::default()
        };
        out.push(ModelPreset {
            id: "auctions-three",
            name: "Three bidders learn together",
            source: "Banchio & Skrzypacz 2022, arXiv 2202.05947v1",
            description: "Starting Q=1/(1-discount), approximately 100 at .99; sampled auction ties, expected hindsight ties. Numerical initialization and tie conventions are our reconstruction. Fixed horizon, final stability selection. Result 8/Figure 11: 500 three-bidder terminal-policy sessions; canvas projects bidders 1 and 2. Native registered FPA mean .706500 (500/500 stable); joint discount comparison is inconclusive.",
            config: ModelConfig::Auctions(c),
        });
    }
    {
        let c = AuctionsConfig {
            bidders: 3,
            discount: 0.999,
            ..Default::default()
        };
        out.push(ModelPreset {
            id: "auctions-three-patient",
            name: "Three bidders put more weight on future rewards",
            source: "Banchio & Skrzypacz 2022, arXiv 2202.05947v1",
            description: "Starting Q=1/(1-discount), approximately 1,000 at .999; sampled auction ties, expected hindsight ties. Numerical initialization and tie conventions are our reconstruction. Fixed horizon, final stability selection. Result 8/Figure 11: 500 three-bidder terminal-policy sessions; initial Q=1/(1-.999), approximately 1,000. Native FPA stability only 34/500, SPA 0/500; high-patience comparison inconclusive, all sessions retained.",
            config: ModelConfig::Auctions(c),
        });
    }
    {
        let c = AuctionsConfig {
            fringe: Fringe::Uniform,
            ..Default::default()
        };
        out.push(ModelPreset {
            id: "auctions-fringe",
            name: "A random bid joins the auction",
            source: "Banchio & Skrzypacz 2022, arXiv 2202.05947v1",
            description: "Starting Q=1/(1-discount), approximately 100 at .99; sampled auction ties, expected hindsight ties. Numerical initialization and tie conventions are our reconstruction. Fixed horizon, final stability selection. Result 9/Figure 8: terminal-policy distribution; source FPA bids .60/.65, compared with symmetric optimum .50. Native FPA mean strategic bid .576727; 48.55% of stable profiles in {.60,.65} misses required 50%, joint check fails.",
            config: ModelConfig::Auctions(c),
        });
    }
    {
        let c = AuctionsConfig {
            exploration: Exploration::Constant,
            epsilon: 0.001,
            horizon: 100_000_000,
            ..Default::default()
        };
        out.push(ModelPreset {
            id: "auctions-persistent",
            name: "Bidders keep experimenting for a hundred million auctions",
            source: "Banchio & Skrzypacz 2022, arXiv 2202.05947v1",
            description: "Starting Q=1/(1-discount), approximately 100 at .99; sampled auction ties, expected hindsight ties. Numerical initialization and tie conventions are our reconstruction. Fixed horizon; final stability is reported separately and does not select occupancy or revenue. Figure 9: one 100m-period session per format; whole-run played occupancy. This long experiment may take substantial time. Native seed-1 100m study: FPA realized revenue .225344, SPA top-pair occupancy .309875; fixed occupancy check fails.",
            config: ModelConfig::Auctions(c),
        });
    }
    out
}
