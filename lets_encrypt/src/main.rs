use clap::{App, Arg, SubCommand};

use dns_online::*;

fn find_entry(
    records: &Vec<Record>,
    name: &str,
    short_name: &str,
    ty: net::DNSType,
    value: Option<&str>,
) -> Option<Record> {
    for record in records {
        if record.record_type != ty || (record.name != name && record.name != short_name) {
            continue;
        }

        if let Some(txt) = value {
            // compare with and without the quotes
            if txt == &record.data
                || (record.data.len() > 2 && txt == &record.data[1..record.data.len() - 1])
            {
                return Some(record.clone());
            }
        } else {
            return Some(record.clone());
        }
    }
    None
}

fn find_entry_in_version(
    domain: &Domain,
    version: &Version,
    name: &str,
    short_name: &str,
    ty: net::DNSType,
    value: Option<&str>,
) -> Option<Record> {
    let zone_entries: Vec<Record> = domain.get_zone_records(&version).unwrap();
    find_entry(&zone_entries, name, short_name, ty, value)
}

fn main() {
    let matches = App::new("le_dns_online")
        .version("0.1")
        .author("Simon Thoby <git+ledns@nightmared.fr>")
        .about(
            "Easily edit records in your online.net DNS zone, handy for generating LE certificates",
        )
        .arg(
            Arg::with_name("API key")
                .short("a")
                .long("api-key")
                .takes_value(true)
                .required(true),
        )
        .arg(
            Arg::with_name("Record")
                .short("n")
                .long("name")
                .takes_value(true)
                .required(true),
        )
        .arg(Arg::with_name("Value").long("value").takes_value(true))
        .arg(
            Arg::with_name("Entry type")
                .short("t")
                .long("type")
                .default_value("TXT")
                .takes_value(true),
        )
        .subcommand(
            SubCommand::with_name("add")
                .about("Add an entry in the DNS zone")
                .arg(
                    Arg::with_name("Version Name")
                        .long("version-name")
                        .takes_value(true)
                        .required(true),
                ),
        )
        .subcommand(
            SubCommand::with_name("delete")
                .about("Delete an entry in the DNS zone")
                .arg(
                    Arg::with_name("Version Name")
                        .long("version-name")
                        .takes_value(true)
                        .required(true),
                ),
        )
        .subcommand(
            SubCommand::with_name("update")
                .about("Modify an entry in the DNS zone")
                .arg(
                    Arg::with_name("New Value")
                        .long("new-value")
                        .takes_value(true)
                        .required(true),
                ),
        )
        .get_matches();

    let api_key = matches.value_of("API key").unwrap();
    let record = {
        let mut record = matches.value_of("Record").unwrap().to_owned();
        if !record.ends_with(".") {
            record.push('.');
        }
        record
    };
    let value = matches.value_of("Value");
    let record_type = matches.value_of("Entry type").unwrap().into();

    let available_domains = match query_available_domains(&api_key) {
        Ok(domain) => domain,
        Err(_) => {
            eprintln!("No domain were found with you api key.");
            return;
        }
    };
    if let Some((domain, local_part)) = Domain::find_and_extract_path(&record, available_domains) {
        let version = domain.get_current_version().unwrap();

        if matches.subcommand_name().is_none() {
            eprintln!("You must specify a subcommand.");
            return;
        }

        let subcommand = matches.subcommand_name().unwrap();

        let old_entry =
            find_entry_in_version(&domain, &version, &record, local_part, record_type, value);

        match subcommand {
            "add" => {
                if value.is_none() {
                    eprintln!("Please specify the value with the --data flag");
                    return;
                }

                if old_entry.is_some() {
                    println!("The entry is already present in the zone, doing nothing.");
                    return;
                }

                println!(
                    "Adding a new record to zone {} for domain {}...",
                    version.name, domain.name
                );

                let version_name = matches
                    .subcommand_matches(subcommand)
                    .map(|x| x.value_of("Version Name"))
                    .unwrap()
                    .unwrap();

                let new_version = domain.duplicate_version(&version, &version_name).unwrap();

                domain
                    .add_record(
                        &new_version,
                        &Record::new(record.clone(), record_type, value.unwrap(), 86400),
                    )
                    .unwrap();

                domain.enable_version(&new_version).unwrap();

                println!("The entry {} has been deployed.", record);
            }
            "delete" => {
                println!(
                    "Suppressing the entries {} in domain {}...",
                    record, domain.name
                );

                if old_entry.is_none() {
                    println!("No such entries found, doing nothing.");
                    return;
                }

                let version_name = matches
                    .subcommand_matches(subcommand)
                    .map(|x| x.value_of("Version Name"))
                    .unwrap()
                    .unwrap();

                let new_version = domain.duplicate_version(&version, &version_name).unwrap();

                domain
                    .delete_record(&new_version, &old_entry.unwrap())
                    .unwrap();

                domain.enable_version(&new_version).unwrap();

                println!("The entry {} has been destroyed.", record);
            }
            "update" => {
                if old_entry.is_none() {
                    println!("No such entries found, cannot update it, doing nothing.");
                    return;
                }

                let new_value = match matches
                    .subcommand_matches(subcommand)
                    .map(|x| x.value_of("New Value"))
                {
                    Some(Some(x)) => x,
                    _ => {
                        eprintln!("Please specify the new value with the --new-value flag");
                        return;
                    }
                };

                if find_entry_in_version(
                    &domain,
                    &version,
                    &record,
                    local_part,
                    record_type,
                    Some(new_value),
                )
                .is_some()
                {
                    println!("The entry is already present in the zone, doing nothing.");
                    return;
                }

                let record = old_entry.unwrap();

                domain
                    .update_current_version_record(&record, new_value)
                    .unwrap();

                println!("The entry {} has been updated.", record.id);
            }
            _ => unreachable!(),
        }
    } else {
        eprintln!("No domain name matching {} found ! Exiting...", record);
    }
}
