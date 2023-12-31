use assert_matches::assert_matches;
use fs_more::{
    error::FileError,
    file::{FileMoveOptions, FileMoveWithProgressOptions, FileProgress},
};
use fs_more_test_harness::{
    assertable::AssertableFilePath,
    error::TestResult,
    trees::{SimpleFileHarness, SimpleTreeHarness},
};

#[test]
pub fn move_file() -> TestResult<()> {
    let harness = SimpleFileHarness::new()?;

    let target_file =
        AssertableFilePath::from_path(harness.test_file.path().with_file_name("test_file2.txt"));

    let file_copy_result: Result<u64, FileError> = fs_more::file::move_file(
        harness.test_file.path(),
        target_file.path(),
        FileMoveOptions {
            overwrite_existing: false,
        },
    );

    assert!(
        file_copy_result.is_ok(),
        "failed to execute move_file: {}",
        file_copy_result.unwrap_err()
    );

    harness.test_file.assert_not_exists();

    target_file.assert_exists();
    target_file.assert_content_matches_expected_value_of_assertable(&harness.test_file);


    harness.destroy()?;
    Ok(())
}

#[test]
pub fn move_file_with_progress() -> TestResult<()> {
    let harness = SimpleFileHarness::new()?;

    let source_file_size_bytes = harness.test_file.path().metadata().unwrap().len();
    let target_file =
        AssertableFilePath::from_path(harness.test_file.path().with_file_name("test_file2.txt"));

    let mut last_progress: Option<FileProgress> = None;

    let file_copy_result: Result<u64, FileError> = fs_more::file::move_file_with_progress(
        harness.test_file.path(),
        target_file.path(),
        FileMoveWithProgressOptions {
            overwrite_existing: false,
            ..Default::default()
        },
        |progress| {
            if let Some(previous_progress) = last_progress.as_ref() {
                assert!(progress.bytes_finished >= previous_progress.bytes_finished);
            }

            last_progress = Some(progress.clone());
        },
    );

    let last_progress = last_progress.unwrap();

    assert_eq!(
        last_progress.bytes_finished,
        source_file_size_bytes
    );
    assert_eq!(last_progress.bytes_total, source_file_size_bytes);

    assert!(
        file_copy_result.is_ok(),
        "failed to execute move_file_with_progress: {}",
        file_copy_result.unwrap_err()
    );

    harness.test_file.assert_not_exists();

    target_file.assert_exists();
    target_file.assert_content_matches_expected_value_of_assertable(&harness.test_file);


    harness.destroy()?;
    Ok(())
}


#[test]
pub fn forbid_move_into_itself() -> TestResult<()> {
    let harness = SimpleFileHarness::new()?;

    let file_move_result: Result<u64, FileError> = fs_more::file::move_file(
        harness.foo_bar.path(),
        harness.foo_bar.path(),
        FileMoveOptions {
            overwrite_existing: false,
        },
    );

    assert!(
        file_move_result.is_err(),
        "move_file should have errored, but got {}.",
        file_move_result.unwrap()
    );

    let move_err = file_move_result.unwrap_err();
    assert_matches!(
        move_err,
        FileError::SourceAndTargetAreTheSameFile,
        "move_file should have errored with \
        SourceAndTargetAreTheSameFile, got {}.",
        move_err
    );

    harness.foo_bar.assert_exists();
    harness.foo_bar.assert_content_unchanged();


    harness.destroy()?;
    Ok(())
}

#[test]
pub fn forbid_move_into_itself_with_overwrite_flag() -> TestResult<()> {
    let harness = SimpleFileHarness::new()?;

    let file_move_result: Result<u64, FileError> = fs_more::file::move_file(
        harness.foo_bar.path(),
        harness.foo_bar.path(),
        FileMoveOptions {
            overwrite_existing: true,
        },
    );

    assert!(
        file_move_result.is_err(),
        "move_file should have errored, but got {}.",
        file_move_result.unwrap()
    );

    let move_err = file_move_result.unwrap_err();
    assert_matches!(
        move_err,
        FileError::SourceAndTargetAreTheSameFile,
        "move_file should have errored with SourceAndTargetAreTheSameFile, got {}.",
        move_err
    );

    harness.foo_bar.assert_exists();
    harness.foo_bar.assert_content_unchanged();


    harness.destroy()?;
    Ok(())
}

#[test]
pub fn forbid_case_insensitive_move_into_itself() -> TestResult<()> {
    let harness = SimpleFileHarness::new()?;

    let upper_case_file_name = harness
        .foo_bar
        .path()
        .file_name()
        .unwrap()
        .to_string_lossy()
        .to_uppercase();

    let target_file =
        AssertableFilePath::from_path(harness.foo_bar.path().with_file_name(upper_case_file_name));

    #[cfg(unix)]
    target_file.assert_not_exists();

    #[cfg(windows)]
    target_file.assert_exists();

    let file_move_result: Result<u64, FileError> = fs_more::file::move_file(
        harness.foo_bar.path(),
        target_file.path(),
        FileMoveOptions {
            overwrite_existing: false,
        },
    );

    #[cfg(unix)]
    {
        assert!(
            file_move_result.is_ok(),
            "move_file should have ok-ed (on unix), but got {}",
            file_move_result.unwrap_err(),
        );

        target_file.assert_exists();
        harness.foo_bar.assert_not_exists();
    }

    #[cfg(windows)]
    {
        assert!(
            file_move_result.is_err(),
            "move_file should have errored, but got {}.",
            file_move_result.unwrap()
        );

        let move_err = file_move_result.unwrap_err();
        assert_matches!(
            move_err,
            FileError::SourceAndTargetAreTheSameFile,
            "move_file should have errored with SourceAndTargetAreTheSameFile, got {}.",
            move_err
        );

        target_file.assert_exists();

        harness.foo_bar.assert_exists();
        harness.foo_bar.assert_content_unchanged();
    }



    harness.destroy()?;
    Ok(())
}


#[test]
pub fn allow_move_overwriting_target_file_with_flag() -> TestResult<()> {
    let harness = SimpleFileHarness::new()?;

    let file_move_result: Result<u64, FileError> = fs_more::file::move_file(
        harness.test_file.path(),
        harness.foo_bar.path(),
        FileMoveOptions {
            overwrite_existing: true,
        },
    );

    assert!(
        file_move_result.is_ok(),
        "move_file should have Ok-ed, but got {}.",
        file_move_result.unwrap_err()
    );

    let move_ok = file_move_result.unwrap();
    assert_eq!(
        harness.test_file.expected_content_unchecked().len(),
        move_ok as usize,
        "move_file did not return the precise amount of moved bytes"
    );

    harness.test_file.assert_not_exists();
    harness.foo_bar.assert_exists();

    harness
        .foo_bar
        .assert_content_matches_expected_value_of_assertable(&harness.test_file);


    harness.destroy()?;
    Ok(())
}


#[test]
pub fn forbid_move_overwriting_target_file_without_flag() -> TestResult<()> {
    let harness = SimpleFileHarness::new()?;

    let file_move_result: Result<u64, FileError> = fs_more::file::move_file(
        harness.test_file.path(),
        harness.foo_bar.path(),
        FileMoveOptions {
            overwrite_existing: false,
        },
    );

    assert!(
        file_move_result.is_err(),
        "move_file should have errored, got {}.",
        file_move_result.unwrap()
    );

    let move_err = file_move_result.unwrap_err();
    assert_matches!(
        move_err,
        FileError::AlreadyExists,
        "move_file should have returned AlreadyExists, got {}",
        move_err
    );

    harness.test_file.assert_exists();
    harness.foo_bar.assert_exists();

    harness.test_file.assert_content_unchanged();
    harness.foo_bar.assert_content_unchanged();


    harness.destroy()?;
    Ok(())
}

/// **On Windows**, creating symbolic links requires administrator privileges, unless Developer mode is enabled.
/// See [https://stackoverflow.com/questions/58038683/allow-mklink-for-a-non-admin-user].
#[test]
pub fn move_file_symlink_behaviour() -> TestResult<()> {
    let harness = SimpleTreeHarness::new()?;

    let symlinked_file = AssertableFilePath::from_path(harness.root.child_path("my-symlink.txt"));
    symlinked_file.assert_not_exists();
    symlinked_file.symlink_to_file(harness.binary_file_a.path())?;
    symlinked_file.assert_is_symlink();

    let real_file_size_in_bytes = symlinked_file.file_size_in_bytes()?;

    let target_file =
        AssertableFilePath::from_path(harness.root.child_path("my-moved-symlink.txt"));
    target_file.assert_not_exists();


    let num_copied_bytes = fs_more::file::move_file(
        symlinked_file.path(),
        target_file.path(),
        FileMoveOptions::default(),
    )
    .unwrap();

    assert_eq!(real_file_size_in_bytes, num_copied_bytes);

    symlinked_file.assert_not_exists();
    harness.binary_file_a.assert_content_unchanged();
    target_file.assert_is_file();

    assert_eq!(
        real_file_size_in_bytes,
        target_file.file_size_in_bytes()?
    );

    harness.destroy()?;
    Ok(())
}

/// **On Windows**, creating symbolic links requires administrator privileges, unless Developer mode is enabled.
/// See [https://stackoverflow.com/questions/58038683/allow-mklink-for-a-non-admin-user].
#[test]
pub fn move_file_with_progress_symlink_behaviour() -> TestResult<()> {
    let harness = SimpleTreeHarness::new()?;

    let symlinked_file = AssertableFilePath::from_path(harness.root.child_path("my-symlink.txt"));
    symlinked_file.assert_not_exists();
    symlinked_file.symlink_to_file(harness.binary_file_a.path())?;
    symlinked_file.assert_is_symlink();

    let real_file_size_in_bytes = symlinked_file.file_size_in_bytes()?;

    let target_file =
        AssertableFilePath::from_path(harness.root.child_path("my-moved-symlink.txt"));
    target_file.assert_not_exists();


    let num_copied_bytes = fs_more::file::move_file_with_progress(
        symlinked_file.path(),
        target_file.path(),
        FileMoveWithProgressOptions::default(),
        |_| {},
    )
    .unwrap();

    assert_eq!(real_file_size_in_bytes, num_copied_bytes);

    symlinked_file.assert_not_exists();
    harness.binary_file_a.assert_content_unchanged();
    target_file.assert_is_file();

    assert_eq!(
        real_file_size_in_bytes,
        target_file.file_size_in_bytes()?
    );

    harness.destroy()?;
    Ok(())
}

#[test]
pub fn forbid_move_file_when_source_is_symlink_to_target() -> TestResult<()> {
    let harness = SimpleFileHarness::new()?;

    let test_symlink =
        AssertableFilePath::from_path(harness.root.child_path("symlink-test-file.txt"));
    test_symlink.assert_not_exists();
    test_symlink
        .symlink_to_file(harness.test_file.path())
        .unwrap();
    test_symlink.assert_is_symlink_to_file();

    let copy_result: Result<u64, FileError> = fs_more::file::move_file(
        test_symlink.path(),
        harness.test_file.path(),
        FileMoveOptions {
            overwrite_existing: true,
        },
    );

    let copy_err = copy_result.unwrap_err();

    match &copy_err {
        FileError::SourceAndTargetAreTheSameFile => {
            // This is the expected error.
        }
        _ => panic!("Unexpected Err: {}", copy_err),
    }


    test_symlink.assert_is_symlink_to_file();
    harness.test_file.assert_is_file();

    harness.destroy()?;
    Ok(())
}


#[test]
pub fn forbid_move_file_with_progress_when_source_is_symlink_to_target() -> TestResult<()> {
    let harness = SimpleFileHarness::new()?;

    let test_symlink =
        AssertableFilePath::from_path(harness.root.child_path("symlink-test-file.txt"));
    test_symlink.assert_not_exists();
    test_symlink
        .symlink_to_file(harness.test_file.path())
        .unwrap();
    test_symlink.assert_is_symlink_to_file();

    let mut last_progress: Option<FileProgress> = None;

    let copy_result: Result<u64, FileError> = fs_more::file::move_file_with_progress(
        test_symlink.path(),
        harness.test_file.path(),
        FileMoveWithProgressOptions {
            overwrite_existing: true,
            ..Default::default()
        },
        |progress| {
            last_progress = Some(progress.clone());
        },
    );

    assert!(last_progress.is_none());

    let copy_err = copy_result.unwrap_err();

    match &copy_err {
        FileError::SourceAndTargetAreTheSameFile => {
            // This is the expected error.
        }
        _ => panic!("Unexpected Err: {}", copy_err),
    }


    test_symlink.assert_is_symlink_to_file();
    harness.test_file.assert_is_file();

    harness.destroy()?;
    Ok(())
}
