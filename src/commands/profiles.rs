use anyhow::Result;
use colored::Colorize;
use serde_json::Value;
use tabled::{Table, Tabled};

use crate::{api::ApiClient, cli::ProfileCommands, display};

#[derive(Table)]
struct ProfileRow {
    #[tabled(rename = "ID")]
    id: String,
    #[tabled(rename = "Name")]
    name: String,
    #[tabled(rename = "Gender")]
    gender: String,
    #[tabled(rename = "Age")]
    age: String,
    #[tabled(rename = "Age Group")]
    age_group: String,
    #[tabled(rename = "Country")]
    country: String,
}

pub async fn run(cmd: ProfileCommands) -> Result<()> {
    let mut client = ApiClient::new()?;

    match cmd {
        ProfileCommands::List {
            gender,
            country,
            age_group,
            min_age,
            max_age,
            sort_by,
            order,
            page,
            limit,
        } => {
            let mut params = vec![
                format!("page={}", page),
                format!("limit={}", limit),
                format!("order={}", order),
            ];
            if let Some(v) = gender { params.push(format!("gender={}", v)); }
            if let Some(v) = country { params.push(format!("country_id={}", v)); }
            if let Some(v) = age_group { params.push(format!("age_group={}", v)); }
            if let Some(v) = min_age { params.push(format!("min_age={}", v)); }
            if let Some(v) = max_age { params.push(format!("max_age={}", v)); }
            if let Some(v) = sort_by { params.push(format!("sort_by={}", v)); }

            let path = format!("/api/profiles?{}", params.join("&"));

            let pb = display::spinner("Fetching profiles...");
            let data: Value = client.get(&path).await?;
            pb.finish_and_clear();

            let profiles = data["data"].as_array().cloned().unwrap_or_default();
            let total = data["total"].as_u64().unwrap_or(0);
            let total_pages = data["total_pages"].as_u64().unwrap_or(1);

            if profiles.is_empty() {
                println!("{}", "No profiles found.".dimmed());
                return Ok(());
            }

            let rows: Vec<ProfileRow> = profiles.iter().map(profile_to_row).collect();
            println!("{}", Table::new(rows));
            println!(
                "\n  {} {} profiles  •  Page {} of {}",
                "→".blue(),
                total,
                page,
                total_pages
            );
        }

        ProfileCommands::Get { id } => {
            let pb = display::spinner("Fetching profile...");
            let data: Value = client.get(&format!("/api/profiles/{}", id)).await?;
            pb.finish_and_clear();

            print_profile_detail(&data);
        }

        ProfileCommands::Search { query } => {
            let encoded = urlencoding::encode(&query);
            let path = format!("/api/profiles/search?q={}", encoded);

            let pb = display::spinner(format!("Searching for \"{}\"...", query).as_str());
            let data: Value = client.get(&path).await?;
            pb.finish_and_clear();

            let profiles = data["data"].as_array().cloned().unwrap_or_default();
            let total = data["total"].as_u64().unwrap_or(0);

            if profiles.is_empty() {
                println!("{}", "No results found.".dimmed());
                return Ok(());
            }

            let rows: Vec<ProfileRow> = profiles.iter().map(profile_to_row).collect();
            println!("{}", Table::new(rows));
            println!("\n  {} {} result(s) for \"{}\"", "→".blue(), total, query);
        }

        ProfileCommands::Create { name } => {
            let pb = display::spinner(format!("Creating profile \"{}\"...", name).as_str());
            let data: Value = client
                .post("/api/profiles", serde_json::json!({ "name": name }))
                .await?;
            pb.finish_and_clear();

            display::success(&format!("Profile created: {}", data["id"].as_str().unwrap_or("")));
            print_profile_detail(&data);
        }

        ProfileCommands::Delete { id } => {
            let pb = display::spinner("Deleting profile...");
            client.delete(&format!("/api/profiles/{}", id)).await?;
            pb.finish_and_clear();

            display::success(&format!("Profile {} deleted.", id));
        }

        ProfileCommands::Export { format, gender, country, age_group } => {
            let mut params = vec![format!("format={}", format)];
            if let Some(v) = gender { params.push(format!("gender={}", v)); }
            if let Some(v) = country { params.push(format!("country_id={}", v)); }
            if let Some(v) = age_group { params.push(format!("age_group={}", v)); }

            let path = format!("/api/profiles/export?{}", params.join("&"));

            let pb = display::spinner("Exporting profiles...");
            let bytes = client.get_bytes(&path).await?;
            pb.finish_and_clear();

            let filename = format!(
                "profiles_{}.{}",
                chrono::Local::now().format("%Y%m%d_%H%M%S"),
                format
            );
            std::fs::write(&filename, bytes)?;
            display::success(&format!("Exported to {}", filename.bold()));
        }
    }

    Ok(())
}

fn profile_to_row(p: &Value) -> ProfileRow {
    ProfileRow {
        id: p["id"].as_str().unwrap_or("—").chars().take(8).collect::<String>() + "...",
        name: p["name"].as_str().unwrap_or("—").to_string(),
        gender: p["gender"].as_str().unwrap_or("—").to_string(),
        age: p["age"].to_string(),
        age_group: p["age_group"].as_str().unwrap_or("—").to_string(),
        country: p["country_name"].as_str().unwrap_or("—").to_string(),
    }
}

fn print_profile_detail(p: &Value) {
    println!();
    let fields = [
        ("ID", p["id"].as_str().unwrap_or("—").to_string()),
        ("Name", p["name"].as_str().unwrap_or("—").to_string()),
        ("Gender", p["gender"].as_str().unwrap_or("—").to_string()),
        (
            "Gender Probability",
            format!("{:.1}%", p["genderProbability"].as_f64().unwrap_or(0.0) * 100.0),
        ),
        ("Age", p["age"].to_string()),
        ("Age Group", p["age_group"].as_str().unwrap_or("—").to_string()),
        ("Country", p["country_name"].as_str().unwrap_or("—").to_string()),
        ("Country ID", p["country_id"].as_str().unwrap_or("—").to_string()),
        (
            "Country Probability",
            format!("{:.1}%", p["country_probability"].as_f64().unwrap_or(0.0) * 100.0),
        ),
        ("Created", p["createdAt"].as_str().unwrap_or("—").to_string()),
    ];

    for (label, value) in &fields {
        println!("  {:25} {}", format!("{}:", label).dimmed(), value.bold());
    }
    println!();
}