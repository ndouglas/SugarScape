//! CSV exports of the statistics history and the current agents.

use std::fmt::Write;

use crate::world::World;

pub fn series_csv(world: &World) -> String {
    let names = crate::stats::series_names(&world.config);
    let mut out = String::from("tick");
    for name in &names {
        out.push(',');
        out.push_str(name);
    }
    out.push('\n');
    for s in world.stats.history() {
        out.push_str(&s.tick.to_string());
        for name in &names {
            write!(out, ",{}", s.value(name).expect("known series")).unwrap();
        }
        out.push('\n');
    }
    out
}

pub fn agents_csv(world: &World) -> String {
    let n = world.config.goods.len();
    let m = world.config.pollution.pollutants.len();
    let mut out = String::from(
        "id,x,y,sex,age,max_age,vision,metabolism,sugar,initial_sugar,tribe,tags,spice,initial_spice,spice_metabolism,foresight,immune,diseases",
    );
    for i in 0..n {
        write!(out, ",holding_{i}").unwrap();
    }
    for k in 0..m {
        write!(out, ",site_pollution_{k}").unwrap();
    }
    out.push('\n');
    for a in world.agents() {
        write!(
            out,
            "{},{},{},{:?},{},{},{},{},{},{},{:?},{},{},{},{},{},{},{}",
            a.id,
            a.pos.x,
            a.pos.y,
            a.sex,
            a.age,
            a.max_age,
            a.vision,
            a.metabolism[0],
            a.holdings[0],
            a.initial[0],
            a.tribe(),
            a.tags.to_bit_string(),
            a.holdings[1],
            a.initial[1],
            a.metabolism[1],
            a.foresight,
            a.immune.to_bit_string(),
            a.diseases
                .iter()
                .map(|d| d.to_string())
                .collect::<Vec<_>>()
                .join(";")
        )
        .unwrap();
        for h in &a.holdings[..n] {
            write!(out, ",{h}").unwrap();
        }
        for p in &world.site(a.pos).pollution[..m] {
            write!(out, ",{p}").unwrap();
        }
        out.push('\n');
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::config::Config;

    #[test]
    fn series_csv_has_a_header_and_a_row_per_tick() {
        let mut w = World::new(Config::default(), 1).unwrap();
        w.run(2);
        let csv = series_csv(&w);
        let lines: Vec<&str> = csv.lines().collect();
        assert_eq!(lines[0], "tick,population,gini,mean_wealth,mean_vision,mean_metabolism,blue_fraction,births,deaths,mean_log_price,sd_log_price,trade_volume,sugar_traded,loans_made,amount_lent,defaults,debt_outstanding,mean_foresight,mean_spice,mean_spice_metabolism,infected_fraction,mean_diseases,diseases_in_circulation,new_infections,trade_pairs,mean_holding_0,mean_metabolism_0,traded_0,mean_pollution_0,group_share_0,group_share_1");
        assert_eq!(lines.len(), 4);
        assert!(lines[1].starts_with("0,400,"));
    }

    #[test]
    fn agents_csv_has_a_row_per_agent() {
        let w = World::new(Config::default(), 1).unwrap();
        let csv = agents_csv(&w);
        assert_eq!(csv.lines().count(), 401);
        let header = csv.lines().next().unwrap();
        assert!(
            header.ends_with(",foresight,immune,diseases,holding_0,site_pollution_0"),
            "{header}"
        );
    }

    #[test]
    fn agents_csv_lists_immune_strings_and_disease_ids() {
        let mut w = crate::testkit::blank_world(5, 5);
        let id = crate::testkit::spawn(&mut w, 0, 0);
        w.agent_mut(id).unwrap().diseases = vec![3, 7];
        let csv = agents_csv(&w);
        let row = csv.lines().nth(1).unwrap();
        assert!(
            row.ends_with(&format!(",{},3;7,10,0", "0".repeat(50))),
            "{row}"
        );
    }
}
