mod arr;
mod config;
mod media_item;
mod overseerr;
mod shared;
mod tautulli;
mod tmdb;
mod utils;

use color_eyre::{eyre::eyre, Report, Result};
use std::{io, process::Command};

use config::Config;
use dialoguer::MultiSelect;
use media_item::{gather_requests_data, CompleteMediaItem};

#[tokio::main]
async fn main() -> Result<()> {
    color_eyre::install()?;

    if let Err(err) = Config::read_conf() {
        return Err(eyre!("Failed to read the config, with the following error: {:?}.\nPlease make sure all fields are filled.", err));
    }

    let (mut requests, errs) = gather_requests_data().await?;

    show_requests_result(&requests, errs)?;

    clear_screen()?;

    let chosen = choose_items_to_delete(&requests)?;

    delete_chosen_items(&mut requests, &chosen).await?;

    Ok(())
}

fn show_requests_result(requests: &Vec<CompleteMediaItem>, errs: Vec<Report>) -> Result<()> {
    show_potential_request_errors(errs)?;

    if requests.len() == 0 {
        println!("You do not seem to have any valid requests, with data available.");
        println!("Are you sure all your requests are available and downloaded? Or some data was unable to be acquired from other services.");
        println!("Either try again later, or look over your requests.");

        println!();
        wait(None)?;
        std::process::exit(0);
    }

    Ok(())
}

fn show_potential_request_errors(errs: Vec<Report>) -> Result<()> {
    if errs.len() == 0 {
        return Ok(());
    }

    println!("You got {} errors while gathering data. Press y to show them, or any other input to continue with the errored items ignored.", errs.len());
    let input = get_user_input()?;
    if !input.starts_with("y") {
        return Ok(());
    }

    errs.iter().enumerate().for_each(|(i, err)| {
        println!("Error {} was {}", i, err);
        print_line();
    });

    println!("Do you want to see the full stack traces? Press y. Otherwise continuing to deletion screen with errored items ignored.");
    let inp = get_user_input()?;
    if inp.starts_with("y") {
        return Ok(());
    }

    errs.iter().enumerate().for_each(|(i, err)| {
        println!("Error {} was {:?}", i + 1, err);
        print_line();
    });

    wait(Some(
        "Press enter to continue to deletion screen with errored items ignored.",
    ))?;

    Ok(())
}

fn choose_items_to_delete(requests: &Vec<CompleteMediaItem>) -> Result<Vec<usize>> {
    clear_screen()?;

    let items_to_show = Config::global().items_shown;
    let chosen: Vec<usize> = MultiSelect::new()
        .with_prompt("Choose what media to delete (SPACE to select, ENTER to confirm selection)")
        .max_length(items_to_show)
        .items(&requests)
        .interact()?;

    if chosen.len() == 0 {
        println!("No items selected. Exiting...");
        std::process::exit(0);
    }

    clear_screen()?;

    verify_chosen(requests, &chosen)?;

    Ok(chosen)
}

fn verify_chosen(requests: &Vec<CompleteMediaItem>, chosen: &Vec<usize>) -> Result<()> {
    println!("Are you sure you want to delete the following items:");
    chosen.iter().for_each(|selection| {
        if let Some(media_item) = requests.get(*selection) {
            let media_type = media_item.media_type;
            println!("- {} - {}", &media_item.title, media_type.to_string());
        } else {
            println!("- Unknown item");
        }
    });

    println!("\ny/n:");
    let user_input = get_user_input()?;

    if !user_input.starts_with("y") {
        println!("Cancelling...");
        std::process::exit(0);
    }

    Ok(())
}

async fn delete_chosen_items(
    requests: &mut Vec<CompleteMediaItem>,
    chosen: &Vec<usize>,
) -> Result<()> {
    let mut errs: Vec<(String, Report)> = Vec::new();

    for selection in chosen.into_iter().rev() {
        let media_item = requests.swap_remove(*selection);
        let title = media_item.title.clone();
        if let Err(err) = media_item.remove_from_server().await {
            errs.push((title, err));
        }
    }

    if errs.len() > 0 {
        println!("Had some errors deleting items:\n");
        errs.iter().for_each(|err| {
            println!(
                "Got the following error while deleting {}: {}",
                err.0, err.0
            );
            print_line();
        });

        wait(None)?;
    }

    Ok(())
}

fn clear_screen() -> Result<()> {
    if cfg!(target_os = "windows") {
        Command::new("cmd").arg("/C").arg("cls").status()?;
        Ok(())
    } else {
        Command::new("clear").status()?;
        Ok(())
    }
}

fn get_user_input() -> Result<String> {
    let mut user_input = String::new();
    let stdin = io::stdin();

    stdin.read_line(&mut user_input)?;

    Ok(user_input.to_lowercase())
}

fn wait(custom_msg: Option<&str>) -> Result<()> {
    if let Some(msg) = custom_msg {
        println!("{}", msg);
    } else {
        println!("Press enter to continue.");
    }
    get_user_input()?;
    Ok(())
}

fn print_line() {
    println!("-----------------------------------------------------------------------------");
}
