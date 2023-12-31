use std::path::PathBuf;

use assert_fs::fixture::FixtureError;
use assert_matches::assert_matches;
use fs_more::{
    error::FileError,
    file::{FileCopyOptions, FileCopyWithProgressOptions, FileProgress},
};
use fs_more_test_harness::{
    assertable::AssertableFilePath,
    error::TestResult,
    trees::{SimpleFileHarness, SimpleTreeHarness},
};


/*
 * COPYING WITHOUT PROGRESS
 */

#[test]
pub fn copy_file() -> TestResult<()> {
    let harness = SimpleFileHarness::new()?;

    let target_file =
        AssertableFilePath::from_path(harness.test_file.path().with_file_name("test_file2.txt"));
    target_file.assert_not_exists();

    let file_copy_result: Result<u64, FileError> = fs_more::file::copy_file(
        harness.test_file.path(),
        target_file.path(),
        FileCopyOptions {
            overwrite_existing: false,
            skip_existing: false,
        },
    );

    assert!(
        file_copy_result.is_ok(),
        "failed to execute copy_file: expected Ok, got {}",
        file_copy_result.unwrap_err()
    );

    harness.test_file.assert_exists();
    harness.test_file.assert_content_unchanged();

    target_file.assert_exists();
    target_file.assert_content_matches_expected_value_of_assertable(&harness.test_file);

    harness.destroy()?;
    Ok(())
}


#[test]
pub fn copy_binary_file() -> TestResult<()> {
    let harness = SimpleTreeHarness::new()?;

    let target_file = AssertableFilePath::from_path(
        harness
            .binary_file_a
            .path()
            .with_file_name("test_file2.txt"),
    );
    target_file.assert_not_exists();

    let file_copy_result = fs_more::file::copy_file(
        harness.binary_file_a.path(),
        target_file.path(),
        FileCopyOptions {
            overwrite_existing: false,
            skip_existing: false,
        },
    );

    assert!(
        file_copy_result.is_ok(),
        "failed to execute copy_file: expected Ok, got {}",
        file_copy_result.unwrap_err()
    );

    harness.binary_file_a.assert_content_unchanged();
    harness.subdirectory_b.assert_exists();
    harness.binary_file_b.assert_content_unchanged();

    target_file.assert_exists();
    target_file.assert_content_matches_expected_value_of_assertable(&harness.binary_file_a);

    harness.destroy()?;
    Ok(())
}


#[test]
pub fn forbid_copy_into_self() -> TestResult<()> {
    let harness = SimpleFileHarness::new()?;

    let file_copy_result = fs_more::file::copy_file(
        harness.test_file.path(),
        harness.test_file.path(),
        FileCopyOptions {
            overwrite_existing: false,
            skip_existing: false,
        },
    );

    assert!(
        file_copy_result.is_err(),
        "copy_file should have errored when trying to copy a file into itself"
    );

    let file_copy_err = file_copy_result.unwrap_err();
    assert_matches!(
        file_copy_err,
        FileError::SourceAndTargetAreTheSameFile,
        "copy_file should have errored with SourceAndTargetAreTheSameFile, got {} instead",
        file_copy_err
    );

    harness.test_file.assert_exists();
    harness.test_file.assert_content_unchanged();


    harness.destroy()?;
    Ok(())
}

#[test]
pub fn case_insensitive_copy_into_self() -> Result<(), FixtureError> {
    let harness = SimpleFileHarness::new()?;

    // Generates an upper-case version of the file name.
    let target_file_name = harness
        .test_file
        .path()
        .file_name()
        .unwrap()
        .to_string_lossy()
        .to_uppercase();

    let target_file =
        AssertableFilePath::from_path(harness.test_file.path().with_file_name(target_file_name));


    let file_copy_result = fs_more::file::copy_file(
        harness.test_file.path(),
        target_file.path(),
        FileCopyOptions {
            overwrite_existing: false,
            skip_existing: false,
        },
    );

    #[cfg(unix)]
    {
        assert!(
            file_copy_result.is_ok(),
            "copy_file should have ok-ed (on unix) when trying to copy a file into itself, \
            even when the case is different"
        );

        assert_eq!(
            file_copy_result.unwrap(),
            harness.test_file.path().metadata().unwrap().len()
        );
    }

    #[cfg(windows)]
    {
        assert!(
            file_copy_result.is_err(),
            "copy_file should have errored (on windows) when trying to copy a file into itself, \
            even when the case is different"
        );

        let file_copy_err = file_copy_result.unwrap_err();
        assert_matches!(
            file_copy_err,
            FileError::SourceAndTargetAreTheSameFile,
            "copy_file should have errored with SourceAndTargetAreTheSameFile, got {} instead",
            file_copy_err
        );
    }


    harness.test_file.assert_exists();
    harness.test_file.assert_content_unchanged();

    harness.destroy()?;
    Ok(())
}

#[test]
pub fn forbid_non_trivial_copy_into_self() -> Result<(), FixtureError> {
    let harness = SimpleTreeHarness::new()?;

    let target_file_path = {
        // Generates an upper-case version of the file name.
        let target_file_name_uppercase = harness
            .binary_file_b
            .path()
            .file_name()
            .unwrap()
            .to_string_lossy()
            .to_uppercase();

        let parent_directory = harness
            .binary_file_b
            .path()
            .parent()
            .expect("Unexpected directory structure.");

        let grandparent_directory = parent_directory
            .parent()
            .expect("Unexpected directory structure.");

        let parent_directory_name = parent_directory
            .file_name()
            .expect("Unexpected directory structure.")
            .to_str()
            .expect("Unexpected directory structure.");

        // Reconstruct a bit more complex version of the same path.
        let non_trivial_subpath = PathBuf::from(format!(
            "{}/../{}/{}",
            parent_directory_name, parent_directory_name, target_file_name_uppercase
        ));

        grandparent_directory.join(non_trivial_subpath)
    };

    let target_file = AssertableFilePath::from_path(target_file_path);

    #[cfg(unix)]
    target_file.assert_not_exists();

    #[cfg(windows)]
    target_file.assert_exists();


    let file_copy_result = fs_more::file::copy_file(
        harness.binary_file_b.path(),
        target_file.path(),
        FileCopyOptions {
            overwrite_existing: false,
            skip_existing: false,
        },
    );

    #[cfg(unix)]
    {
        assert!(
            file_copy_result.is_ok(),
            "copy_file should have ok-ed (on unix) when trying to copy a file into itself \
            even when the case is different and the path includes .."
        );

        assert_eq!(
            file_copy_result.unwrap(),
            harness.binary_file_b.path().metadata().unwrap().len()
        );

        target_file.assert_exists();
    }

    #[cfg(windows)]
    {
        assert!(
            file_copy_result.is_err(),
            "copy_file should have errored when trying to copy a file into itself, \
            even when the case is different and the path includes .."
        );

        let file_copy_err = file_copy_result.unwrap_err();
        assert_matches!(
            file_copy_err,
            FileError::SourceAndTargetAreTheSameFile,
            "copy_file should have errored with SourceAndTargetAreTheSameFile, got {} instead",
            file_copy_err
        );

        target_file.assert_exists();
    }


    harness.binary_file_b.assert_exists();
    harness.binary_file_b.assert_content_unchanged();


    harness.destroy()?;
    Ok(())
}

#[test]
pub fn allow_copy_overwriting_file_with_flag() -> TestResult<()> {
    let harness = SimpleFileHarness::new()?;

    let file_copy_result = fs_more::file::copy_file(
        harness.test_file.path(),
        harness.foo_bar.path(),
        FileCopyOptions {
            overwrite_existing: true,
            skip_existing: false,
        },
    );

    assert!(
        file_copy_result.is_ok(),
        "Failed to execute copy_file: expected Ok, got {}",
        file_copy_result.unwrap_err()
    );


    harness.foo_bar.assert_exists();
    harness.test_file.assert_exists();
    harness.test_file.assert_content_unchanged();

    harness
        .foo_bar
        .assert_content_matches_expected_value_of_assertable(&harness.test_file);


    harness.destroy()?;
    Ok(())
}

#[test]
pub fn forbid_copy_overwriting_file_without_flag() -> TestResult<()> {
    let harness = SimpleFileHarness::new()?;

    let file_copy_result = fs_more::file::copy_file(
        harness.test_file.path(),
        harness.foo_bar.path(),
        FileCopyOptions {
            overwrite_existing: false,
            skip_existing: false,
        },
    );

    assert!(
        file_copy_result.is_err(),
        "copy_file returned {} instead of Err",
        file_copy_result.unwrap()
    );

    harness.test_file.assert_exists();
    harness.foo_bar.assert_exists();

    harness.test_file.assert_content_unchanged();
    harness.foo_bar.assert_content_unchanged();


    harness.destroy()?;
    Ok(())
}

#[test]
pub fn skip_existing_target_file_move_with_flag() -> TestResult<()> {
    let harness = SimpleFileHarness::new()?;

    let file_copy_result = fs_more::file::copy_file(
        harness.test_file.path(),
        harness.foo_bar.path(),
        FileCopyOptions {
            overwrite_existing: false,
            skip_existing: true,
        },
    );

    assert!(
        file_copy_result.is_ok(),
        "copy_file returned {} instead of Ok",
        file_copy_result.unwrap()
    );
    assert_eq!(
        file_copy_result.unwrap(),
        0,
        "copy_file returned Ok, but copied non-zero bytes",
    );

    harness.test_file.assert_exists();
    harness.foo_bar.assert_exists();

    harness.test_file.assert_content_unchanged();
    harness.foo_bar.assert_content_unchanged();


    harness.destroy()?;
    Ok(())
}



/*
 * COPYING WITH PROGRESS
 */


#[test]
pub fn copy_file_with_progress() -> TestResult<()> {
    let harness = SimpleFileHarness::new()?;

    let target_file =
        AssertableFilePath::from_path(harness.test_file.path().with_file_name("test_file2.txt"));
    target_file.assert_not_exists();

    let expected_final_file_size_bytes = harness.test_file.path().metadata()?.len();

    let mut last_bytes_copied = 0;
    let mut total_bytes = 0;

    let file_copy_result: Result<u64, FileError> = fs_more::file::copy_file_with_progress(
        harness.test_file.path(),
        target_file.path(),
        FileCopyWithProgressOptions {
            overwrite_existing: false,
            skip_existing: false,
            ..Default::default()
        },
        |progress| {
            last_bytes_copied = progress.bytes_finished;
            total_bytes = progress.bytes_total;
        },
    );

    assert!(
        file_copy_result.is_ok(),
        "failed to execute copy_file_with_progress: expected Ok, got {}.",
        file_copy_result.unwrap_err()
    );

    let bytes_copied = file_copy_result.unwrap();
    assert_eq!(
        bytes_copied, last_bytes_copied,
        "copy_file_with_progress failed to report some last writes \
        (return value and last progress update do not match)"
    );

    assert_eq!(
        bytes_copied, expected_final_file_size_bytes,
        "copied vs real total bytes mismatch: the copy_file_with_progress return value \
        doesn't match the entire file size reported by the filesystem"
    );
    assert_eq!(
        bytes_copied, total_bytes,
        "copied vs total bytes mismatch: the copy_file_with_progress return value \
        doesn't match the entire file size reported by the same function"
    );

    harness.test_file.assert_exists();
    harness.test_file.assert_content_unchanged();

    target_file.assert_exists();
    target_file.assert_content_matches_expected_value_of_assertable(&harness.test_file);


    harness.destroy()?;
    Ok(())
}


/// **On Windows**, creating symbolic links requires administrator privileges, unless Developer mode is enabled.
/// See [https://stackoverflow.com/questions/58038683/allow-mklink-for-a-non-admin-user].
#[test]
pub fn copy_file_symlink_behaviour() -> TestResult<()> {
    let harness = SimpleTreeHarness::new()?;

    let symlinked_file = AssertableFilePath::from_path(harness.root.child_path("my-symlink.txt"));
    symlinked_file.assert_not_exists();
    symlinked_file.symlink_to_file(harness.binary_file_a.path())?;
    symlinked_file.assert_is_symlink();

    let real_file_size_in_bytes = symlinked_file.file_size_in_bytes()?;

    let target_file =
        AssertableFilePath::from_path(harness.root.child_path("my-copied-symlink.txt"));
    target_file.assert_not_exists();


    let num_copied_bytes = fs_more::file::copy_file(
        symlinked_file.path(),
        target_file.path(),
        FileCopyOptions::default(),
    )
    .unwrap();

    assert_eq!(real_file_size_in_bytes, num_copied_bytes);

    symlinked_file.assert_is_symlink();
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
pub fn copy_file_with_progress_symlink_behaviour() -> TestResult<()> {
    let harness = SimpleTreeHarness::new()?;

    let symlinked_file = AssertableFilePath::from_path(harness.root.child_path("my-symlink.txt"));
    symlinked_file.assert_not_exists();
    symlinked_file.symlink_to_file(harness.binary_file_a.path())?;
    symlinked_file.assert_is_symlink_to_file();

    let real_file_size_in_bytes = symlinked_file.file_size_in_bytes()?;

    let target_file =
        AssertableFilePath::from_path(harness.root.child_path("my-copied-symlink.txt"));
    target_file.assert_not_exists();


    let num_copied_bytes = fs_more::file::copy_file_with_progress(
        symlinked_file.path(),
        target_file.path(),
        FileCopyWithProgressOptions::default(),
        |_| {},
    )
    .unwrap();

    assert_eq!(real_file_size_in_bytes, num_copied_bytes);

    symlinked_file.assert_is_symlink_to_file();
    target_file.assert_is_file();

    assert_eq!(
        real_file_size_in_bytes,
        target_file.file_size_in_bytes()?
    );

    harness.destroy()?;
    Ok(())
}


#[test]
pub fn forbid_copy_file_when_source_is_symlink_to_target() -> TestResult<()> {
    // Tests behaviour when copying "symlink to file A" to "A".
    // This should fail.

    let harness = SimpleFileHarness::new()?;


    let test_symlink =
        AssertableFilePath::from_path(harness.root.child_path("symlink-test-file.txt"));
    test_symlink.assert_not_exists();
    test_symlink
        .symlink_to_file(harness.test_file.path())
        .unwrap();
    test_symlink.assert_is_symlink_to_file();

    let copy_result: Result<u64, FileError> = fs_more::file::copy_file(
        test_symlink.path(),
        harness.test_file.path(),
        FileCopyOptions {
            overwrite_existing: true,
            skip_existing: false,
        },
    );

    let copy_err = copy_result.unwrap_err();

    match &copy_err {
        FileError::SourceAndTargetAreTheSameFile => {
            // This is the expected behaviour in this case.
        }
        _ => panic!("Unexpected Err: {}", copy_err),
    }


    harness.destroy()?;

    Ok(())
}


#[test]
pub fn forbid_copy_file_with_progress_when_source_is_symlink_to_target() -> TestResult<()> {
    // Tests behaviour when copying "symlink to file A" to "A".
    // This should fail.

    let harness = SimpleFileHarness::new()?;

    let test_symlink =
        AssertableFilePath::from_path(harness.root.child_path("symlink-test-file.txt"));
    test_symlink.assert_not_exists();
    test_symlink
        .symlink_to_file(harness.test_file.path())
        .unwrap();
    test_symlink.assert_is_symlink_to_file();


    let mut last_progress: Option<FileProgress> = None;

    let copy_result: Result<u64, FileError> = fs_more::file::copy_file_with_progress(
        test_symlink.path(),
        harness.test_file.path(),
        FileCopyWithProgressOptions {
            overwrite_existing: true,
            skip_existing: false,
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
            // This is the expected behaviour in this case.
        }
        _ => panic!("Unexpected Err: {}", copy_err),
    }


    harness.destroy()?;

    Ok(())
}
