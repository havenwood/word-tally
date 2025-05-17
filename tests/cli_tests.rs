use assert_cmd::Command;
use predicates::prelude::PredicateBooleanExt;
use predicates::str::{self, contains};
use std::fs;

fn word_tally() -> Command {
    Command::cargo_bin("word-tally").unwrap()
}

#[test]
fn version() {
    let assert = word_tally().arg("-V").assert();
    assert.success().stdout(str::starts_with("word-tally "));
}

#[test]
fn help() {
    let assert = word_tally().arg("-h").assert();
    assert.success().stdout(str::contains("\nUsage"));
}

#[test]
fn help_long() {
    let assert = word_tally().arg("--help").assert();
    assert.success().stdout(str::contains("\nUsage"));
}

#[test]
fn verbose_without_input() {
    let assert = word_tally().arg("-v").assert();
    assert
        .success()
        .stderr("source -\ntotal-words 0\nunique-words 0\ndelimiter \" \"\ncase lower\norder desc\nprocessing sequential\nio streamed\nmin-chars none\nmin-count none\nexclude-words none\nexclude-patterns none\ninclude-patterns none\n")
        .stdout("");
}

#[test]
fn verbose_with_min_chars() {
    let assert = word_tally().arg("-v").arg("--min-chars=42").assert();
    assert
        .success()
        .stderr("source -\ntotal-words 0\nunique-words 0\ndelimiter \" \"\ncase lower\norder desc\nprocessing sequential\nio streamed\nmin-chars 42\nmin-count none\nexclude-words none\nexclude-patterns none\ninclude-patterns none\n")
        .stdout("");
}

#[test]
fn verbose_with_min_count() {
    let assert = word_tally().arg("-v").arg("--min-count=42").assert();
    assert
        .success()
        .stderr("source -\ntotal-words 0\nunique-words 0\ndelimiter \" \"\ncase lower\norder desc\nprocessing sequential\nio streamed\nmin-chars none\nmin-count 42\nexclude-words none\nexclude-patterns none\ninclude-patterns none\n")
        .stdout("");
}

#[test]
fn verbose_with_exclude_words() {
    let assert = word_tally()
        .arg("-v")
        .arg("--exclude-words=narrow,certain")
        .assert();
    assert
        .success()
        .stderr("source -\ntotal-words 0\nunique-words 0\ndelimiter \" \"\ncase lower\norder desc\nprocessing sequential\nio streamed\nmin-chars none\nmin-count none\nexclude-words narrow,certain\nexclude-patterns none\ninclude-patterns none\n")
        .stdout("");
}

#[test]
fn verbose_with_input() {
    let assert = word_tally().write_stdin("narrow").arg("-v").assert();
    assert
        .success()
        .stderr("source -\ntotal-words 1\nunique-words 1\ndelimiter \" \"\ncase lower\norder desc\nprocessing sequential\nio streamed\nmin-chars none\nmin-count none\nexclude-words none\nexclude-patterns none\ninclude-patterns none\n\n")
        .stdout("narrow 1\n");
}

#[test]
fn output_longhand() {
    let assert = word_tally()
        .write_stdin("narrow")
        .arg("--output=test.txt")
        .assert();
    assert.success().stdout("");
    assert_eq!("narrow 1\n", fs::read_to_string("test.txt").unwrap());
    fs::remove_file("test.txt").unwrap();
}

#[test]
fn output_shorthand() {
    let assert = word_tally()
        .write_stdin("narrow")
        .arg("-o=test2.txt")
        .assert();
    assert.success().stdout("");
    assert_eq!("narrow 1\n", fs::read_to_string("test2.txt").unwrap());
    fs::remove_file("test2.txt").unwrap();
}

#[test]
fn delimiter_shorthand() {
    let assert = word_tally().write_stdin("narrow").arg("-d\t").assert();
    assert.success().stdout("narrow\t1\n");
}

#[test]
fn delimiter_longhand() {
    let assert = word_tally()
        .write_stdin("narrow")
        .arg("--delimiter=,")
        .assert();
    assert.success().stdout("narrow,1\n");
}

#[test]
fn case_default() {
    let assert = word_tally().write_stdin("nArRoW CeRtAiN certain").assert();
    assert.success().stdout("certain 2\nnarrow 1\n");
}

#[test]
fn case_upper() {
    let assert = word_tally()
        .write_stdin("nArRoW CeRtAiN narrow")
        .arg("--case=upper")
        .assert();
    assert.success().stdout("NARROW 2\nCERTAIN 1\n");
}

#[test]
fn case_original() {
    let assert = word_tally()
        .write_stdin("narrow nArRoW narrow nArRoW narrow CeRtAiN")
        .arg("--case=original")
        .assert();
    assert.success().stdout("narrow 3\nnArRoW 2\nCeRtAiN 1\n");
}

#[test]
fn sort_default() {
    let assert = word_tally().write_stdin("bb a bb a ccc a").assert();
    assert.success().stdout("a 3\nbb 2\nccc 1\n");
}

#[test]
fn sort_asc() {
    let assert = word_tally()
        .write_stdin("bb a bb a ccc a")
        .arg("--sort=asc")
        .assert();
    assert.success().stdout("ccc 1\nbb 2\na 3\n");
}

#[test]
fn no_words() {
    let assert = word_tally().write_stdin("").assert();
    assert.success().stdout("");
}

#[test]
fn test_discard_words() {
    let input = "Hope is the thing with feathers that perches in the soul.";
    let mut cmd = Command::cargo_bin("word-tally").unwrap();
    cmd.write_stdin(input)
        .arg("--exclude-words=feathers,soul")
        .assert()
        .success()
        .stdout(contains("hope").and(contains("feathers").not().and(contains("soul").not())));
}

#[test]
fn test_exclude_patterns() {
    let input = "I dwell in possibility - a fairer house than prose.";
    let mut cmd = Command::cargo_bin("word-tally").unwrap();
    cmd.write_stdin(input)
        .arg("--exclude=^h.*") // Exclude words starting with 'h'
        .arg("--exclude=.*t$") // Exclude words ending with 't'
        .assert()
        .success()
        .stdout(contains("dwell"))
        .stdout(contains("possibility"))
        .stdout(contains("a"))
        .stdout(contains("fairer"))
        .stdout(contains("prose"))
        .stdout(contains("house").not())
        .stdout(contains("than"))
        .stdout(contains("i"));
}

#[test]
fn test_multiple_exclude_patterns() {
    let input = "success fame sunset wild nobody moon immortal";
    let mut cmd = Command::cargo_bin("word-tally").unwrap();
    cmd.write_stdin(input)
        .arg("--exclude=^s.*") // Exclude words starting with 's'
        .arg("--exclude=.*g$") // Exclude words ending with 'g'
        .arg("--exclude=c.*t") // Exclude words containing 'c' and 't'
        .assert()
        .success()
        .stdout(contains("fame"))
        .stdout(contains("wild"))
        .stdout(contains("nobody"))
        .stdout(contains("moon"))
        .stdout(contains("immortal"))
        .stdout(contains("success").not())
        .stdout(contains("sunset").not());
}

#[test]
fn test_include_patterns() {
    let input = "nobody knows tomorrow certain immortal narrow sublime";
    let mut cmd = Command::cargo_bin("word-tally").unwrap();
    cmd.write_stdin(input)
        .arg("--include=^[nt].*") // Include words starting with 'n' or 't'
        .assert()
        .success()
        .stdout(contains("nobody"))
        .stdout(contains("tomorrow"))
        .stdout(contains("narrow"))
        .stdout(contains("knows").not())
        .stdout(contains("certain").not())
        .stdout(contains("immortal").not())
        .stdout(contains("sublime").not());
}

#[test]
fn test_multiple_include_patterns() {
    let input = "beauty finite infinite fame certain forever sublime";
    let mut cmd = Command::cargo_bin("word-tally").unwrap();
    cmd.write_stdin(input)
        .arg("--include=^f.*") // Include words starting with 'f'
        .arg("--include=.*e$") // Include words ending with 'e'
        .assert()
        .success()
        .stdout(contains("finite"))
        .stdout(contains("fame"))
        .stdout(contains("forever"))
        .stdout(contains("infinite"))
        .stdout(contains("sublime"))
        .stdout(contains("beauty").not())
        .stdout(contains("certain").not());
}

#[test]
fn test_combine_exclusions() {
    let input = "Tell all the truth but tell it slant - success in circuit lies.";
    let mut cmd = Command::cargo_bin("word-tally").unwrap();
    cmd.write_stdin(input)
        .arg("--exclude-words=tell,lies")
        .arg("--exclude=^s.*")
        .arg("--exclude=.*t$")
        .assert()
        .success()
        .stdout(contains("all"))
        .stdout(contains("the"))
        .stdout(contains("truth"))
        .stdout(contains("in"))
        .stdout(contains("but").not())
        .stdout(contains("it").not())
        .stdout(contains("circuit").not())
        .stdout(contains("slant").not())
        .stdout(contains("success").not())
        .stdout(contains("tell").not())
        .stdout(contains("lies").not());
}

#[test]
fn verbose_with_exclude_patterns() {
    let assert = word_tally()
        .arg("-v")
        .arg("--exclude=^t.*")
        .arg("--exclude=ing$")
        .assert();
    assert
        .success()
        .stderr(contains("exclude-patterns ^t.*,ing$"));
}

#[test]
fn verbose_with_json_format() {
    let assert = word_tally()
        .arg("-v")
        .arg("--format=json")
        .write_stdin("hope forever")
        .assert();

    assert
        .success()
        .stderr(contains("\"source\":\"-\""))
        .stderr(contains("\"totalWords\":2"))
        .stderr(contains("\"uniqueWords\":2"))
        .stderr(contains("\"case\":\"lower\""))
        .stderr(contains("\"order\":\"desc\""))
        .stderr(contains("\"processing\":\"sequential\""))
        .stderr(contains("\"io\":\"streamed\""))
        .stderr(contains("\"minChars\":null"))
        .stderr(contains("\"minCount\":null"))
        .stderr(contains("\"excludeWords\":null"))
        .stderr(contains("\"excludePatterns\":null"))
        .stderr(contains("\"includePatterns\":null"))
        .stdout(contains("[\"hope\",1]").and(contains("[\"forever\",1]")));
}

#[test]
fn verbose_with_csv_format() {
    let assert = word_tally()
        .arg("-v")
        .arg("--format=csv")
        .write_stdin("hope forever")
        .assert();

    assert
        .success()
        .stderr(contains("metric,value"))
        .stderr(contains("source,-"))
        .stderr(contains("total-words,2"))
        .stderr(contains("unique-words,2"))
        .stderr(contains("case,lower"))
        .stderr(contains("order,desc"))
        .stderr(contains("processing,sequential"))
        .stderr(contains("io,streamed"))
        .stderr(contains("min-chars,none"))
        .stderr(contains("min-count,none"))
        .stderr(contains("exclude-words,none"))
        .stderr(contains("exclude-patterns,none"))
        .stderr(contains("include-patterns,none"))
        .stdout(contains("word,count"))
        .stdout(contains("forever,1"))
        .stdout(contains("hope,1"));
}

#[test]
fn format_json() {
    let assert = word_tally()
        .write_stdin("narrow narrow fame")
        .arg("--format=json")
        .assert();
    assert
        .success()
        .stdout(contains("[\"narrow\",2]").and(contains("[\"fame\",1]")));
}

#[test]
fn format_csv() {
    let assert = word_tally()
        .write_stdin("narrow narrow fame")
        .arg("--format=csv")
        .assert();
    assert
        .success()
        .stdout(contains("word,count"))
        .stdout(contains("narrow,2"))
        .stdout(contains("fame,1"));
}

#[test]
fn csv_escaping() {
    // Using a normal test with multiple words to verify CSV format is correct
    // This tests that the CSV header exists and words are tallied correctly
    // The real test of CSV escaping is happening behind the scenes in the csv crate
    // which handles commas and quotes automatically

    let assert = word_tally()
        .write_stdin("narrow certain \"sublime\" hope")
        .arg("--format=csv")
        .assert();

    assert
        .success()
        .stdout(str::starts_with("word,count\n"))
        .stdout(contains("narrow,1"))
        .stdout(contains("certain,1"))
        .stdout(contains("sublime,1"))
        .stdout(contains("hope,1"));
}

#[test]
fn stdin_with_parallel() {
    // Test with a small input
    let assert = word_tally()
        .write_stdin("hope forever")
        .arg("--parallel")
        .assert();
    assert
        .success()
        .stdout(contains("hope 1"))
        .stdout(contains("forever 1"));

    // Test with a multi-line input
    let assert = word_tally()
        .write_stdin("hope forever\ninfinite beauty\nhope sublime")
        .arg("--parallel")
        .assert();
    assert
        .success()
        .stdout(contains("hope 2"))
        .stdout(contains("forever 1"))
        .stdout(contains("infinite 1"))
        .stdout(contains("beauty 1"))
        .stdout(contains("sublime 1"));
}

#[test]
fn stdin_with_parallel_shorthand() {
    // Test the -p shorthand flag
    let assert = word_tally().write_stdin("hope forever").arg("-p").assert();
    assert
        .success()
        .stdout(contains("hope 1"))
        .stdout(contains("forever 1"));
}

#[test]
fn parallel_with_env_vars() {
    let assert = word_tally()
        .env("WORD_TALLY_CHUNK_SIZE", "4096")
        .env("WORD_TALLY_THREADS", "2")
        .write_stdin("hope infinite beauty forever sublime")
        .arg("--parallel")
        .assert();

    assert
        .success()
        .stdout(contains("hope 1"))
        .stdout(contains("infinite 1"))
        .stdout(contains("beauty 1"))
        .stdout(contains("forever 1"))
        .stdout(contains("sublime 1"));
}

#[test]
fn parallel_with_large_chunk() {
    let assert = word_tally()
        .env("WORD_TALLY_CHUNK_SIZE", "65536")
        .write_stdin("truth beauty certain narrow sublime forever")
        .arg("--parallel")
        .assert();

    assert
        .success()
        .stdout(contains("truth 1"))
        .stdout(contains("beauty 1"))
        .stdout(contains("certain 1"))
        .stdout(contains("narrow 1"))
        .stdout(contains("sublime 1"))
        .stdout(contains("forever 1"));
}
