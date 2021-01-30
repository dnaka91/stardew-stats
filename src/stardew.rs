use std::{
    collections::HashMap,
    convert::{TryFrom, TryInto},
    str::FromStr,
    u64,
};

use anyhow::{anyhow, bail, ensure, Context, Result};
use roxmltree::Node;

fn get<'a>(value: &'a HashMap<&'a str, Node<'a, 'a>>, name: &str) -> Result<&'a Node<'a, 'a>> {
    value
        .get(name)
        .with_context(|| anyhow!("field `{}` is missing", name))
}

fn get_string(value: &HashMap<&str, Node<'_, '_>>, name: &str) -> Result<String> {
    get(value, name)?
        .text()
        .with_context(|| anyhow!("field `{}` has not content", name))
        .map(ToOwned::to_owned)
}

fn get_bool(value: &HashMap<&str, Node<'_, '_>>, name: &str) -> Result<bool> {
    Ok(get(value, name)?
        .text()
        .with_context(|| anyhow!("field `{}` has not content", name))?
        == "true")
}

fn parse<T, E>(value: &HashMap<&str, Node<'_, '_>>, name: &str) -> Result<T>
where
    T: FromStr<Err = E>,
    E: Into<anyhow::Error>,
{
    let content = get(value, name)?
        .text()
        .with_context(|| anyhow!("field `{}` has not content", name))?;
    content
        .parse()
        .map_err(Into::into)
        .with_context(|| anyhow!("invalid content `{}` in `{}` field", content, name))
}

fn try_into<'a, T, E>(value: &'a HashMap<&str, Node<'_, '_>>, name: &str) -> Result<T>
where
    T: TryFrom<HashMap<&'a str, Node<'a, 'a>>, Error = E>,
    E: Into<anyhow::Error>,
{
    get(value, name)?
        .children()
        .filter(|c| c.is_element())
        .map(|c| (c.tag_name().name(), c))
        .collect::<HashMap<_, _>>()
        .try_into()
        .map_err(Into::into)
}

fn try_into_list<'a, T, E>(
    value: &'a HashMap<&str, Node<'_, '_>>,
    name: &str,
    tag: &str,
) -> Result<Vec<T>>
where
    T: TryFrom<(Option<&'a str>, HashMap<&'a str, Node<'a, 'a>>), Error = E>,
    E: Into<anyhow::Error>,
{
    get(value, name)?
        .children()
        .filter(|c| {
            c.is_element()
                && !c
                    .attribute(("http://www.w3.org/2001/XMLSchema-instance", "nil"))
                    .map(|a| a == "true")
                    .unwrap_or_default()
        })
        .map(|c| {
            let name = c.tag_name().name();
            ensure!(name == tag, "tag name wasn't `{}` but `{}`", tag, name);

            let ty = c.attribute(("http://www.w3.org/2001/XMLSchema-instance", "type"));
            let map = c
                .children()
                .filter(|c| c.is_element())
                .map(|c| (c.tag_name().name(), c))
                .collect::<HashMap<_, _>>();

            (ty, map).try_into().map_err(Into::into)
        })
        .collect()
}

fn get_list<T, F>(
    value: &HashMap<&str, Node<'_, '_>>,
    name: &str,
    tag: &str,
    transform: F,
) -> Result<Vec<T>>
where
    F: Fn(&str) -> Result<T>,
{
    get(value, name)?
        .children()
        .filter(|c| c.is_element())
        .map(|c| {
            let name = c.tag_name().name();
            ensure!(name == tag, "tag name wasn't `{}` but `{}`", tag, name);
            let value = c
                .text()
                .with_context(|| anyhow!("no content in <{}> tag", tag))?;

            transform(value).map_err(Into::into)
        })
        .collect()
}

fn get_int_list(value: &HashMap<&str, Node<'_, '_>>, name: &str) -> Result<Vec<u64>> {
    get_list(value, name, "int", |value| {
        value.parse().map_err(Into::into)
    })
}

fn get_string_list_with_tag(
    value: &HashMap<&str, Node<'_, '_>>,
    name: &str,
    tag: &str,
) -> Result<Vec<String>> {
    get_list(value, name, tag, |value| Ok(value.to_owned()))
}

fn get_string_list(value: &HashMap<&str, Node<'_, '_>>, name: &str) -> Result<Vec<String>> {
    get_string_list_with_tag(value, name, "string")
}

#[derive(Debug)]
pub struct SaveGame {
    pub player: Player,
    // TODO: implement
    locations: (),
    pub current_season: Season,
    pub sam_band_name: String,
    pub elliott_book_name: String,
    // TODO: implement
    broadcasted_mail: (),
    pub world_state_ids: Vec<String>,
    pub lost_books_found: u64,
    pub day_of_month: u8,
    pub year: u32,
    pub farmer_wallpaper: u64,
    pub farmer_floor: u64,
    pub current_wallpaper: u64,
    pub current_floor: u64,
    pub current_song_index: u64,
    countdown_to_wedding: (),
    pub incubating_egg: Position,
    pub chance_to_rain_tomorrow: f64,
    pub daily_luck: f64,
    pub unique_id_for_this_game: String,
    pub wedding_today: bool,
    pub is_raining: bool,
    pub is_debris_weather: bool,
    pub shipping_tax: bool,
    pub bloom_day: bool,
    pub is_lightning: bool,
    pub is_snowing: bool,
    pub should_spawn_monsters: bool,
    pub has_applied_1_3_update_changes: bool,
    pub has_applied_1_4_update_changes: bool,
    pub music_volume: f64,
    pub sound_volume: f64,
    pub crops_of_the_week: Vec<u64>,
    dis_of_the_day: (),
    pub highest_player_limit: u8,
    pub move_building_permission_mode: u64,
    banned_users: (),
    pub latest_id: i64,
    custom_data: (),
    mine_permanent_mine_changes: (),
    pub mine_lowest_level_reached: u8,
    pub minecart_high_score: u64,
    pub weather_for_tomorrow: u64,
    pub which_farm: u8,
    junimo_cart_leaderboards: (),
    farmer_friendships: (),
    cellar_assignments: (),
    pub last_applied_save_fix: u64,
    pub game_version: String,
}

impl<'a> TryFrom<&HashMap<&'a str, Node<'a, 'a>>> for SaveGame {
    type Error = anyhow::Error;

    fn try_from(value: &HashMap<&'a str, Node<'a, 'a>>) -> Result<Self, Self::Error> {
        Ok(Self {
            player: try_into(&value, "player")?,
            locations: (),
            current_season: parse(value, "currentSeason")?,
            sam_band_name: get_string(value, "samBandName")?,
            elliott_book_name: get_string(value, "elliottBookName")?,
            broadcasted_mail: (),
            world_state_ids: get_string_list(value, "worldStateIDs")?,
            lost_books_found: parse(value, "lostBooksFound")?,
            day_of_month: parse(value, "dayOfMonth")?,
            year: parse(value, "year")?,
            farmer_wallpaper: parse(value, "farmerWallpaper")?,
            farmer_floor: parse(value, "FarmerFloor")?,
            current_wallpaper: parse(value, "currentWallpaper")?,
            current_floor: parse(value, "currentFloor")?,
            current_song_index: parse(value, "currentSongIndex")?,
            countdown_to_wedding: (),
            incubating_egg: try_into(value, "incubatingEgg")?,
            chance_to_rain_tomorrow: parse(value, "chanceToRainTomorrow")?,
            daily_luck: parse(value, "dailyLuck")?,
            unique_id_for_this_game: get_string(value, "uniqueIDForThisGame")?,
            wedding_today: get_bool(value, "weddingToday")?,
            is_raining: get_bool(value, "isRaining")?,
            is_debris_weather: get_bool(value, "isDebrisWeather")?,
            shipping_tax: get_bool(value, "shippingTax")?,
            bloom_day: get_bool(value, "bloomDay")?,
            is_lightning: get_bool(value, "isLightning")?,
            is_snowing: get_bool(value, "isSnowing")?,
            should_spawn_monsters: get_bool(value, "shouldSpawnMonsters")?,
            has_applied_1_3_update_changes: get_bool(value, "hasApplied1_3_UpdateChanges")?,
            has_applied_1_4_update_changes: get_bool(value, "hasApplied1_4_UpdateChanges")?,
            music_volume: parse(value, "musicVolume")?,
            sound_volume: parse(value, "soundVolume")?,
            crops_of_the_week: get_int_list(value, "cropsOfTheWeek")?,
            dis_of_the_day: (),
            highest_player_limit: parse(value, "highestPlayerLimit")?,
            move_building_permission_mode: parse(value, "moveBuildingPermissionMode")?,
            banned_users: (),
            latest_id: parse(value, "latestID")?,
            custom_data: (),
            mine_permanent_mine_changes: (),
            mine_lowest_level_reached: parse(value, "mine_lowestLevelReached")?,
            minecart_high_score: parse(value, "minecartHighScore")?,
            weather_for_tomorrow: parse(value, "weatherForTomorrow")?,
            which_farm: parse(value, "whichFarm")?,
            junimo_cart_leaderboards: (),
            farmer_friendships: (),
            cellar_assignments: (),
            last_applied_save_fix: parse(value, "lastAppliedSaveFix")?,
            game_version: get_string(value, "gameVersion")?,
        })
    }
}

#[derive(Debug)]
pub enum Season {
    Spring,
    Summer,
    Autumn,
    Winter,
}

impl FromStr for Season {
    type Err = anyhow::Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Ok(match s {
            "spring" => Self::Spring,
            "summer" => Self::Summer,
            "autumn" => Self::Autumn,
            "winter" => Self::Winter,
            _ => bail!("unknown season `{}`", s),
        })
    }
}

#[derive(Debug)]
pub struct Player {
    pub name: String,
    pub is_emoting: bool,
    pub is_charging: bool,
    pub is_glowing: bool,
    pub colored_border: bool,
    pub flip: bool,
    pub draw_on_top: bool,
    pub face_toward_farmer: bool,
    pub ignore_movement_animation: bool,
    pub face_away_from_farmer: bool,
    pub scale: f32,
    pub time_before_ai_movement_again: u64,
    pub glowing_transparency: f64,
    pub glow_rate: f64,
    pub will_destroy_objects_underfoot: bool,
    pub position: Position,
    pub speed: u64,
    pub facing_direction: u64,
    pub is_emoting2: bool,
    pub current_emote: u64,
    pub scale2: f64,
    // TODO: implement quest log parsing
    // quest_log: QuestLog,
    pub professions: Vec<u64>,
    new_levels: (), // TODO: Don't know the type yet
    pub experience_points: Vec<u64>,
    pub items: Vec<Item>,
    pub dialogue_questions_answered: Vec<u64>,
    furniture_owned: (), // TODO: Don't know the type yet
    // TODO: implement cooking recipe parsing
    // cooking_recipes: Vec<(String, u64)>,
    // TODO: implement crafting recipe parsing
    // crafting_recipes: Vec<(String, u64)>,
    active_dialogue_events: (), // TODO: Don't know the type yet
    pub events_seen: Vec<u64>,
    secret_notes_seen: (), // TODO: Don't know the type yet
    pub songs_heard: Vec<String>,
    pub achievements: Vec<u64>,
    pub special_items: Vec<u64>,
    pub special_big_craftables: Vec<u64>,
    pub mail_received: Vec<String>,
    mail_for_tomorrow: (), // TODO: Don't know the type yet
    pub mailbox: Vec<String>,
    pub time_went_to_bed: u64,
    // TODO: implement stats parsing
    // stats: Stats,
    blueprints: (), // TODO: Don't know the type yet
    // TODO: implement item parsing
    // items_lost_last_death: Vec<Item>,
    pub farm_name: String,
    pub favorite_thing: String,
    pub slot_can_host: bool,
    user_id: (), // TODO: Don't know the type yet
    pub cat_person: bool,
    pub which_pet_breed: u64,
    pub accepted_daily_quest: bool,
    pub most_recent_bed: Position,
    performed_emotes: (), // TODO: Don't know the type yet
    pub shirt: i64,
    pub hair: u64,
    pub skin: u64,
    pub shoes: i64,
    pub accessory: i64,
    pub facial_hair: i64,
    pub pants: i64,
    pub hairstyle_color: Color,
    pub pants_color: Color,
    pub new_eye_color: Color,
    pub shirt_item: ClothingItem,
    pub pants_item: ClothingItem,
    pub divorce_tonight: bool,
    pub change_wallet_type_tonight: bool,
    pub wood_pieces: u64,
    pub stone_pieces: u64,
    pub copper_pieces: u64,
    pub iron_pieces: u64,
    pub coal_pieces: u64,
    pub gold_pieces: u64,
    pub iridium_pieces: u64,
    pub quartz_pieces: u64,
    pub game_version: String,
    pub cave_choice: u8,
    pub feed: u64,
    pub farming_level: u8,
    pub mining_level: u8,
    pub combat_level: u8,
    pub foraging_level: u8,
    pub fishing_level: u8,
    pub luck_level: u8,
    pub new_skill_points_to_spend: u8,
    pub added_farming_level: u8,
    pub added_mining_level: u8,
    pub added_combat_level: u8,
    pub added_foraging_level: u8,
    pub added_fishing_level: u8,
    pub added_luck_level: u8,
    pub max_stamina: u32,
    pub max_items: u32,
    pub last_seen_movie_week: i32,
    pub resilience: u64,
    pub attack: u64,
    pub immunity: u64,
    pub attack_increase_modifier: u64,
    pub knockback_modifier: u64,
    pub weapon_speed_modifier: u64,
    pub crit_chance_modifier: u64,
    pub crit_power_modifier: u64,
    pub weapon_precision_modifier: u64,
    pub club_coins: u64,
    pub trash_can_level: u8,
    // TODO: implement tool item parsing
    // pub tool_being_upgraded: ToolItem,
    pub days_left_for_tool_upgrade: u8,
    pub house_upgrade_level: u8,
    pub days_until_house_upgrade: i8,
    pub coop_upgrade_level: u8,
    pub barn_upgrade_level: u8,
    pub has_greenhouse: bool,
    pub has_unlocked_skull_door: bool,
    pub has_dark_talisman: bool,
    pub has_magic_ink: bool,
    pub show_chest_color_picker: bool,
    pub has_magnifying_glass: bool,
    pub has_watering_can_enchantment: bool,
    pub magnetic_radius: u16,
    pub temporary_invincibility_timer: u64,
    pub health: u32,
    pub max_health: u32,
    pub difficulty_modifier: i32,
    pub is_male: bool,
    pub has_bus_ticket: bool,
    pub stardew_hero: bool,
    pub has_club_card: bool,
    pub has_special_charm: bool,
    // TODO: implement
    // basic_shipped: Vec<()>,
    // minerals_found: Vec<()>,
    // recipes_cooked: Vec<()>,
    // fish_caught: Vec<()>,
    // archaelogy_found: Vec<()>,
    // gifted_items: Vec<()>,
    // tailored_items: Vec<()>,
    // friendship_data: Vec<()>,
    pub day_of_month_for_save_game: u8,
    pub season_for_save_game: u8,
    pub year_for_safe_game: u32,
    pub overalls_color: u32,
    pub shirt_color: u32,
    pub skin_color: u32,
    pub hair_color: u32,
    pub eye_color: u32,
    // TODO: implement
    // bobber: (),
    // chest_consumed_levels: (),
    pub save_time: u64,
    pub is_customized: bool,
    pub home_location: String,
    pub days_married: u64,
    pub movement_multiplier: f64,
    pub theater_build_date: i64,
    pub deepest_mine_level: u8,
    pub stamina: u32,
    pub total_money_earned: u64,
    pub milliseconds_played: u64,
    pub has_rusty_key: bool,
    pub has_skull_key: bool,
    pub can_understand_dwarves: bool,
    pub use_separate_wallets: bool,
    pub times_reached_mine_bottom: u64,
    pub unique_multiplayer_id: String,
    pub money: u64,
}

impl<'a> TryFrom<HashMap<&'a str, Node<'a, 'a>>> for Player {
    type Error = anyhow::Error;

    fn try_from(value: HashMap<&'a str, Node<'a, 'a>>) -> Result<Self, Self::Error> {
        Ok(Self {
            name: get_string(&value, "name")?,
            is_emoting: get_bool(&value, "isEmoting")?,
            is_charging: get_bool(&value, "isCharging")?,
            is_glowing: get_bool(&value, "isGlowing")?,
            colored_border: get_bool(&value, "coloredBorder")?,
            flip: get_bool(&value, "flip")?,
            draw_on_top: get_bool(&value, "drawOnTop")?,
            face_toward_farmer: get_bool(&value, "faceTowardFarmer")?,
            ignore_movement_animation: get_bool(&value, "ignoreMovementAnimation")?,
            face_away_from_farmer: get_bool(&value, "faceAwayFromFarmer")?,
            scale: value["scale"]
                .first_element_child()
                .unwrap()
                .text()
                .unwrap()
                .parse()?,
            time_before_ai_movement_again: parse(&value, "timeBeforeAIMovementAgain")?,
            glowing_transparency: parse(&value, "glowingTransparency")?,
            glow_rate: parse(&value, "glowRate")?,
            will_destroy_objects_underfoot: get_bool(&value, "willDestroyObjectsUnderfoot")?,
            position: try_into(&value, "Position")?,
            speed: parse(&value, "Speed")?,
            facing_direction: parse(&value, "FacingDirection")?,
            is_emoting2: get_bool(&value, "IsEmoting")?,
            current_emote: parse(&value, "CurrentEmote")?,
            scale2: parse(&value, "Scale")?,
            professions: get_int_list(&value, "professions")?,
            new_levels: (),
            experience_points: get_int_list(&value, "experiencePoints")?,
            items: try_into_list(&value, "items", "Item")?,
            dialogue_questions_answered: get_int_list(&value, "dialogueQuestionsAnswered")?,
            furniture_owned: (),
            active_dialogue_events: (),
            events_seen: get_int_list(&value, "eventsSeen")?,
            secret_notes_seen: (),
            songs_heard: get_string_list(&value, "songsHeard")?,
            achievements: get_int_list(&value, "achievements")?,
            special_items: get_int_list(&value, "specialItems")?,
            special_big_craftables: get_int_list(&value, "specialBigCraftables")?,
            mail_received: get_string_list(&value, "mailReceived")?,
            mail_for_tomorrow: (),
            mailbox: get_string_list(&value, "mailbox")?,
            time_went_to_bed: value["timeWentToBed"]
                .first_element_child()
                .unwrap()
                .text()
                .unwrap()
                .parse()?,
            blueprints: (),
            farm_name: get_string(&value, "farmName")?,
            favorite_thing: get_string(&value, "favoriteThing")?,
            slot_can_host: get_bool(&value, "slotCanHost")?,
            user_id: (),
            cat_person: get_bool(&value, "catPerson")?,
            which_pet_breed: parse(&value, "whichPetBreed")?,
            accepted_daily_quest: get_bool(&value, "acceptedDailyQuest")?,
            most_recent_bed: try_into(&value, "mostRecentBed")?,
            performed_emotes: (),
            shirt: parse(&value, "shirt")?,
            hair: parse(&value, "hair")?,
            skin: parse(&value, "skin")?,
            shoes: parse(&value, "shoes")?,
            accessory: parse(&value, "accessory")?,
            facial_hair: parse(&value, "facialHair")?,
            pants: parse(&value, "pants")?,
            hairstyle_color: try_into(&value, "hairstyleColor")?,
            pants_color: try_into(&value, "pantsColor")?,
            new_eye_color: try_into(&value, "newEyeColor")?,
            shirt_item: try_into(&value, "shirtItem")?,
            pants_item: try_into(&value, "pantsItem")?,
            divorce_tonight: get_bool(&value, "divorceTonight")?,
            change_wallet_type_tonight: get_bool(&value, "changeWalletTypeTonight")?,
            wood_pieces: parse(&value, "woodPieces")?,
            stone_pieces: parse(&value, "stonePieces")?,
            copper_pieces: parse(&value, "copperPieces")?,
            iron_pieces: parse(&value, "ironPieces")?,
            coal_pieces: parse(&value, "coalPieces")?,
            gold_pieces: parse(&value, "goldPieces")?,
            iridium_pieces: parse(&value, "iridiumPieces")?,
            quartz_pieces: parse(&value, "quartzPieces")?,
            game_version: get_string(&value, "gameVersion")?,
            cave_choice: parse(&value, "caveChoice")?,
            feed: parse(&value, "feed")?,
            farming_level: parse(&value, "farmingLevel")?,
            mining_level: parse(&value, "miningLevel")?,
            combat_level: parse(&value, "combatLevel")?,
            foraging_level: parse(&value, "foragingLevel")?,
            fishing_level: parse(&value, "fishingLevel")?,
            luck_level: parse(&value, "luckLevel")?,
            new_skill_points_to_spend: parse(&value, "newSkillPointsToSpend")?,
            added_farming_level: parse(&value, "addedFarmingLevel")?,
            added_mining_level: parse(&value, "addedMiningLevel")?,
            added_combat_level: parse(&value, "addedCombatLevel")?,
            added_foraging_level: parse(&value, "addedForagingLevel")?,
            added_fishing_level: parse(&value, "addedFishingLevel")?,
            added_luck_level: parse(&value, "addedLuckLevel")?,
            max_stamina: parse(&value, "maxStamina")?,
            max_items: parse(&value, "maxItems")?,
            last_seen_movie_week: parse(&value, "lastSeenMovieWeek")?,
            resilience: parse(&value, "resilience")?,
            attack: parse(&value, "attack")?,
            immunity: parse(&value, "immunity")?,
            attack_increase_modifier: parse(&value, "attackIncreaseModifier")?,
            knockback_modifier: parse(&value, "knockbackModifier")?,
            weapon_speed_modifier: parse(&value, "weaponSpeedModifier")?,
            crit_chance_modifier: parse(&value, "critChanceModifier")?,
            crit_power_modifier: parse(&value, "critPowerModifier")?,
            weapon_precision_modifier: parse(&value, "weaponPrecisionModifier")?,
            club_coins: parse(&value, "clubCoins")?,
            trash_can_level: parse(&value, "trashCanLevel")?,
            days_left_for_tool_upgrade: parse(&value, "daysLeftForToolUpgrade")?,
            house_upgrade_level: parse(&value, "houseUpgradeLevel")?,
            days_until_house_upgrade: parse(&value, "daysUntilHouseUpgrade")?,
            coop_upgrade_level: parse(&value, "coopUpgradeLevel")?,
            barn_upgrade_level: parse(&value, "barnUpgradeLevel")?,
            has_greenhouse: get_bool(&value, "hasGreenhouse")?,
            has_unlocked_skull_door: get_bool(&value, "hasUnlockedSkullDoor")?,
            has_dark_talisman: get_bool(&value, "hasDarkTalisman")?,
            has_magic_ink: get_bool(&value, "hasMagicInk")?,
            show_chest_color_picker: get_bool(&value, "showChestColorPicker")?,
            has_magnifying_glass: get_bool(&value, "hasMagnifyingGlass")?,
            has_watering_can_enchantment: get_bool(&value, "hasWateringCanEnchantment")?,
            magnetic_radius: parse(&value, "magneticRadius")?,
            temporary_invincibility_timer: parse(&value, "temporaryInvincibilityTimer")?,
            health: parse(&value, "health")?,
            max_health: parse(&value, "maxHealth")?,
            difficulty_modifier: parse(&value, "difficultyModifier")?,
            is_male: get_bool(&value, "isMale")?,
            has_bus_ticket: get_bool(&value, "hasBusTicket")?,
            stardew_hero: get_bool(&value, "stardewHero")?,
            has_club_card: get_bool(&value, "hasClubCard")?,
            has_special_charm: get_bool(&value, "hasSpecialCharm")?,
            day_of_month_for_save_game: parse(&value, "dayOfMonthForSaveGame")?,
            season_for_save_game: parse(&value, "seasonForSaveGame")?,
            year_for_safe_game: parse(&value, "yearForSaveGame")?,
            overalls_color: parse(&value, "overallsColor")?,
            shirt_color: parse(&value, "shirtColor")?,
            skin_color: parse(&value, "skinColor")?,
            hair_color: parse(&value, "hairColor")?,
            eye_color: parse(&value, "eyeColor")?,
            save_time: parse(&value, "saveTime")?,
            is_customized: get_bool(&value, "isCustomized")?,
            home_location: get_string(&value, "homeLocation")?,
            days_married: parse(&value, "daysMarried")?,
            movement_multiplier: parse(&value, "movementMultiplier")?,
            theater_build_date: parse(&value, "theaterBuildDate")?,
            deepest_mine_level: parse(&value, "deepestMineLevel")?,
            stamina: parse(&value, "stamina")?,
            total_money_earned: parse(&value, "totalMoneyEarned")?,
            milliseconds_played: parse(&value, "millisecondsPlayed")?,
            has_rusty_key: get_bool(&value, "hasRustyKey")?,
            has_skull_key: get_bool(&value, "hasSkullKey")?,
            can_understand_dwarves: get_bool(&value, "canUnderstandDwarves")?,
            use_separate_wallets: get_bool(&value, "useSeparateWallets")?,
            times_reached_mine_bottom: parse(&value, "timesReachedMineBottom")?,
            unique_multiplayer_id: get_string(&value, "UniqueMultiplayerID")?,
            money: parse(&value, "money")?,
        })
    }
}

#[derive(Debug, Default)]
pub struct Position {
    pub x: f64,
    pub y: f64,
}

impl<'a> TryFrom<HashMap<&'a str, Node<'a, 'a>>> for Position {
    type Error = anyhow::Error;

    fn try_from(value: HashMap<&'a str, Node<'a, 'a>>) -> Result<Self, Self::Error> {
        Ok(Self {
            x: parse(&value, "X")?,
            y: parse(&value, "Y")?,
        })
    }
}

#[derive(Debug)]
struct QuestLog {
    quest: Vec<Quest>,
}

#[derive(Debug)]
struct Quest {
    current_objective: String,
    quest_description: String,
    quest_title: String,
    accepted: bool,
    completed: bool,
    daily_quest: bool,
    show_new: bool,
    can_be_cancelled: bool,
    destroy: bool,
    id: u64,
    money_reward: u64,
    quest_type: u64,
    days_left: u8,
    day_quest_accepted: i64,
    next_quests: Vec<u64>,
    ty: Option<QuestType>,
}

#[derive(Debug)]
enum QuestType {
    ItemDeliveryQuest {
        // target_message: String,
    // target: String,
    // item: u64,
    // number: u64,
    // deliver_item: Option<DeliveryItem>,
    // parts: Vec<()>,
    // dialogueparts: Vec<()>,
    // objective: Option<Objective>,
    },
    LostItemQuest {
        // npc_name: String,
    // location_of_item: String,
    // item_index: u64,
    // tile_x: u64,
    // tile_y: u64,
    // item_found: bool,
    },
}

#[derive(Debug, Default)]
pub struct Color {
    pub r: u8,
    pub g: u8,
    pub b: u8,
    pub a: u8,
    pub packed_value: u32,
}

impl<'a> TryFrom<HashMap<&'a str, Node<'a, 'a>>> for Color {
    type Error = anyhow::Error;

    fn try_from(value: HashMap<&'a str, Node<'a, 'a>>) -> Result<Self, Self::Error> {
        Ok(Self {
            r: parse(&value, "R")?,
            g: parse(&value, "G")?,
            b: parse(&value, "B")?,
            a: parse(&value, "A")?,
            packed_value: parse(&value, "PackedValue")?,
        })
    }
}

#[derive(Debug)]
pub struct ClothingItem {
    pub is_lost_item: bool,
    pub category: i64,
    pub has_been_in_inventory: bool,
    pub name: String,
    pub parent_sheet_index: u64,
    pub special_item: bool,
    pub special_variable: i64,
    pub display_name: String,
    pub name2: String,
    pub stack: u64,
    pub price: u64,
    pub index_in_tile_sheet: u64,
    pub index_in_tile_sheet_female: i64,
    pub clothes_type: u64,
    pub dyeable: bool,
    pub clothes_color: Color,
    pub other_data: (), // TODO: Don't know the type yet
    pub is_prismatic: bool,
    pub price2: u64,
}

impl<'a> TryFrom<HashMap<&'a str, Node<'a, 'a>>> for ClothingItem {
    type Error = anyhow::Error;

    fn try_from(value: HashMap<&'a str, Node<'a, 'a>>) -> Result<Self, Self::Error> {
        Ok(Self {
            is_lost_item: get_bool(&value, "isLostItem")?,
            category: parse(&value, "category")?,
            has_been_in_inventory: get_bool(&value, "hasBeenInInventory")?,
            name: get_string(&value, "name")?,
            parent_sheet_index: parse(&value, "parentSheetIndex")?,
            special_item: get_bool(&value, "specialItem")?,
            special_variable: parse(&value, "SpecialVariable")?,
            display_name: get_string(&value, "DisplayName")?,
            name2: get_string(&value, "Name")?,
            stack: parse(&value, "Stack")?,
            price: parse(&value, "price")?,
            index_in_tile_sheet: parse(&value, "indexInTileSheet")?,
            index_in_tile_sheet_female: parse(&value, "indexInTileSheetFemale")?,
            clothes_type: parse(&value, "clothesType")?,
            dyeable: get_bool(&value, "dyeable")?,
            clothes_color: try_into(&value, "clothesColor")?,
            other_data: (),
            is_prismatic: get_bool(&value, "isPrismatic")?,
            price2: parse(&value, "Price")?,
        })
    }
}

#[derive(Debug)]
pub struct Item {
    pub is_lost_item: bool,
    pub category: i64,
    pub has_been_in_inventory: bool,
    pub name: String,
    pub special_item: bool,
    pub special_variable: i64,
    pub display_name: String,
    pub name2: String,
    pub stack: u64,
}

impl TryFrom<(Option<&str>, HashMap<&str, Node<'_, '_>>)> for Item {
    type Error = anyhow::Error;

    fn try_from(
        (_ty, value): (Option<&str>, HashMap<&str, Node<'_, '_>>),
    ) -> Result<Self, Self::Error> {
        Ok(Self {
            is_lost_item: get_bool(&value, "isLostItem")?,
            category: parse(&value, "category")?,
            has_been_in_inventory: get_bool(&value, "hasBeenInInventory")?,
            name: get_string(&value, "name")?,
            special_item: get_bool(&value, "specialItem")?,
            special_variable: parse(&value, "SpecialVariable")?,
            display_name: get_string(&value, "DisplayName")?,
            name2: get_string(&value, "Name")?,
            stack: parse(&value, "Stack")?,
        })
    }
}

pub fn load(file: &str) -> Result<SaveGame> {
    let doc = roxmltree::Document::parse(file)?;

    SaveGame::try_from(
        &doc.root_element()
            .children()
            .filter(|c| c.is_element())
            .map(|c| (c.tag_name().name(), c))
            .collect(),
    )
}
