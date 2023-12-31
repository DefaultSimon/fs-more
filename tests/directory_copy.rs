use assert_matches::assert_matches;
use fs_more::{
    directory::{
        DirectoryCopyOptions,
        DirectoryCopyProgress,
        DirectoryCopyWithProgressOptions,
        DirectoryScan,
        TargetDirectoryRule,
    },
    error::DirectoryError,
    file::FileCopyOptions,
};
use fs_more_test_harness::{
    assertable::{AssertableDirectoryPath, AssertableFilePath},
    error::TestResult,
    trees::{DeepTreeHarness, EmptyTreeHarness},
};

#[test]
pub fn copy_directory() -> TestResult<()> {
    let harness = DeepTreeHarness::new()?;
    let empty_harness = EmptyTreeHarness::new()?;

    let source_scan = DirectoryScan::scan_with_options(harness.root.path(), None, false)
        .expect("failed to scan temporary directory");
    let source_full_size = source_scan
        .total_size_in_bytes()
        .expect("failed to compute size of source directory in bytes");

    empty_harness.root.assert_is_empty();

    let finished_copy = fs_more::directory::copy_directory(
        harness.root.path(),
        empty_harness.root.path(),
        DirectoryCopyOptions {
            target_directory_rule: TargetDirectoryRule::AllowEmpty,
            ..Default::default()
        },
    )
    .unwrap_or_else(|error| {
        panic!(
            "copy_directory unexpectedly failed with Err: {}",
            error
        );
    });


    assert_eq!(
        source_full_size, finished_copy.total_bytes_copied,
        "DirectoryScan and copy_directory report different amount of bytes"
    );

    assert_eq!(
        source_scan.files.len(),
        finished_copy.num_files_copied,
        "DirectoryScan and copy_directory report different number of files"
    );

    assert_eq!(
        source_scan.directories.len(),
        finished_copy.num_directories_created,
        "DirectoryScan and copy_directory report different number of directories"
    );

    harness
        .root
        .assert_directory_contents_match_directory(empty_harness.root.path());


    harness.destroy()?;
    empty_harness.destroy()?;
    Ok(())
}


#[test]
pub fn copy_directory_respect_maximum_depth_option() -> TestResult<()> {
    let harness = DeepTreeHarness::new()?;
    let empty_harness = EmptyTreeHarness::new()?;

    const MAXIMUM_DEPTH: Option<usize> = Some(2);

    let source_scan = DirectoryScan::scan_with_options(harness.root.path(), MAXIMUM_DEPTH, false)
        .expect("failed to scan temporary directory");
    let source_full_size = source_scan
        .total_size_in_bytes()
        .expect("failed to compute size of source directory in bytes");

    empty_harness.root.assert_is_empty();

    let finished_copy = fs_more::directory::copy_directory(
        harness.root.path(),
        empty_harness.root.path(),
        DirectoryCopyOptions {
            target_directory_rule: TargetDirectoryRule::AllowEmpty,
            maximum_copy_depth: MAXIMUM_DEPTH,
        },
    )
    .unwrap_or_else(|error| {
        panic!(
            "copy_directory unexpectedly failed with Err: {}",
            error
        );
    });

    assert_eq!(
        source_full_size, finished_copy.total_bytes_copied,
        "DirectoryScan and copy_directory report different amount of bytes"
    );

    assert_eq!(
        source_scan.files.len(),
        finished_copy.num_files_copied,
        "DirectoryScan and copy_directory report different number of files"
    );

    assert_eq!(
        source_scan.directories.len(),
        finished_copy.num_directories_created,
        "DirectoryScan and copy_directory report different number of directories"
    );

    harness.destroy()?;
    empty_harness.destroy()?;
    Ok(())
}



#[test]
pub fn copy_directory_with_progress() -> TestResult<()> {
    let harness = DeepTreeHarness::new()?;
    let empty_harness = EmptyTreeHarness::new()?;

    let source_scan = DirectoryScan::scan_with_options(harness.root.path(), None, false)
        .expect("failed to scan temporary directory");
    let source_full_size = source_scan
        .total_size_in_bytes()
        .expect("failed to compute size of source directory in bytes");

    empty_harness.root.assert_is_empty();

    let mut last_progress: Option<DirectoryCopyProgress> = None;

    let finished_copy = fs_more::directory::copy_directory_with_progress(
        harness.root.path(),
        empty_harness.root.path(),
        DirectoryCopyWithProgressOptions {
            target_directory_rule: TargetDirectoryRule::AllowEmpty,
            ..Default::default()
        },
        |progress| {
            if last_progress.is_none() {
                last_progress = Some(progress.clone());
                return;
            };

            let previous_progress = last_progress.as_ref().unwrap();
            let progress_operation_index_delta = progress.current_operation_index - previous_progress.current_operation_index;

            if progress_operation_index_delta != 0 && progress_operation_index_delta != 1 {
                panic!(
                    "copy_directory_with_progress reported non-consecutive operation indexes: {} and {}",
                    previous_progress.current_operation_index,
                    progress.current_operation_index
                );
            }

            assert!(
                progress.current_operation_index >= 0,
                "copy_directory_with_progress reported a negative operation index: {}",
                progress.current_operation_index
            );

            last_progress = Some(progress.clone());
        },
    )
    .unwrap_or_else(|error| {
        panic!(
            "copy_directory_with_progress unexpectedly failed with Err: {}",
            error
        );
    });


    assert!(
        last_progress.is_some(),
        "copy_directory_with_progress did not report progress at all"
    );

    let last_progress = last_progress.unwrap();

    assert_eq!(
        last_progress.current_operation_index + 1,
        last_progress.total_operations,
        "copy_directory_with_progress's last progress reported inconsistent operation indexes"
    );

    assert_eq!(
        last_progress.bytes_finished, last_progress.bytes_total,
        "copy_directory_with_progress's last progress message was an unfinished copy"
    );
    assert_eq!(
        source_full_size,
        last_progress.bytes_total,
        "DirectoryScan and copy_directory_with_progress's last progress reported different amount of total bytes"
    );
    assert_eq!(
        source_full_size, finished_copy.total_bytes_copied,
        "DirectoryScan and copy_directory_with_progress report different amount of total bytes"
    );

    assert_eq!(
        source_scan.files.len(),
        last_progress.files_copied,
        "copy_directory_with_progress's last progress did not report all files"
    );
    assert_eq!(
        source_scan.files.len(),
        finished_copy.num_files_copied,
        "DirectoryScan and copy_directory_with_progress report different number of files"
    );

    assert_eq!(
        source_scan.directories.len(),
        last_progress.directories_created,
        "copy_directory_with_progress's last progress did not report all directories"
    );
    assert_eq!(
        source_scan.directories.len(),
        finished_copy.num_directories_created,
        "DirectoryScan and copy_directory_with_progress report different number of directories"
    );

    harness
        .root
        .assert_directory_contents_match_directory(empty_harness.root.path());


    harness.destroy()?;
    empty_harness.destroy()?;
    Ok(())
}


#[test]
pub fn copy_directory_with_progress_respect_depth_option() -> TestResult<()> {
    let harness = DeepTreeHarness::new()?;
    let empty_harness = EmptyTreeHarness::new()?;

    const MAXIMUM_DEPTH: Option<usize> = Some(2);

    let source_scan = DirectoryScan::scan_with_options(harness.root.path(), MAXIMUM_DEPTH, false)
        .expect("failed to scan temporary directory");
    let source_full_size = source_scan
        .total_size_in_bytes()
        .expect("failed to compute size of source directory in bytes");

    empty_harness.root.assert_is_empty();

    fs_more::directory::copy_directory_with_progress(
        harness.root.path(),
        empty_harness.root.path(),
        DirectoryCopyWithProgressOptions {
            target_directory_rule: TargetDirectoryRule::AllowEmpty,
            maximum_copy_depth: MAXIMUM_DEPTH,
            ..Default::default()
        },
        |_| {},
    )
    .unwrap_or_else(|error| {
        panic!(
            "copy_directory_with_progress unexpectedly failed with Err: {}",
            error
        );
    });

    let target_scan = DirectoryScan::scan_with_options(empty_harness.root.path(), None, false)
        .expect("failed to scan target temporary directory");
    let target_full_size = target_scan
        .total_size_in_bytes()
        .expect("failed to compute size of target directory in bytes");

    assert_eq!(
        source_full_size, target_full_size,
        "copy_directory_with_progress did not create an equally-sized directory copy"
    );

    harness.destroy()?;
    empty_harness.destroy()?;
    Ok(())
}


#[test]
pub fn error_on_copy_directory_with_progress_on_existing_file_without_option() -> TestResult<()> {
    let harness = DeepTreeHarness::new()?;
    let empty_harness = EmptyTreeHarness::new()?;

    // Still the harness setup.
    let file_a_filename = harness.file_a.path().file_name().unwrap();
    let test_file_path = empty_harness.root.child_path(file_a_filename);
    fs_more::file::copy_file(
        harness.file_a.path(),
        &test_file_path,
        FileCopyOptions {
            overwrite_existing: false,
            skip_existing: false,
        },
    )
    .unwrap();

    let test_file = AssertableFilePath::from_path_with_captured_content(test_file_path)?;

    test_file.assert_exists();
    test_file.assert_content_unchanged();
    // End of setup, we have now pre-copied a single file to test our overwriting options.


    empty_harness.root.assert_is_not_empty();

    let copy_result = fs_more::directory::copy_directory_with_progress(
        harness.root.path(),
        empty_harness.root.path(),
        DirectoryCopyWithProgressOptions {
            target_directory_rule: TargetDirectoryRule::AllowNonEmpty {
                overwrite_existing_files: false,
                overwrite_existing_subdirectories: true,
            },
            ..Default::default()
        },
        |_| {},
    );

    assert!(
        copy_result.is_err(),
        "copy_directory_with_progress should have errored due to existing target file"
    );

    let copy_err = copy_result.unwrap_err();
    match &copy_err {
        DirectoryError::TargetItemAlreadyExists { path } => {
            assert_eq!(
                path,
                test_file.path(),
                "copy_directory_with_progress returned TargetItemAlreadyExists with incorrect inner path"
            );
        }
        _ => {
            panic!("copy_directory_with_progress should have errored with TargetItemAlreadyExists")
        }
    }


    harness.destroy()?;
    empty_harness.destroy()?;
    Ok(())
}

#[test]
pub fn error_on_copy_directory_with_progress_on_existing_directory_without_option() -> TestResult<()>
{
    let harness = DeepTreeHarness::new()?;
    let empty_harness = EmptyTreeHarness::new()?;

    // Still the harness setup.
    let replicated_foo_dir_name = harness.dir_foo.path().file_name().unwrap();
    let replicated_foo_dir_path = empty_harness.root.child_path(replicated_foo_dir_name);
    std::fs::create_dir_all(&replicated_foo_dir_path)?;
    // End of setup, we have now pre-copied a single file to test our overwriting options.


    empty_harness.root.assert_is_not_empty();

    let copy_result = fs_more::directory::copy_directory_with_progress(
        harness.root.path(),
        empty_harness.root.path(),
        DirectoryCopyWithProgressOptions {
            target_directory_rule: TargetDirectoryRule::AllowNonEmpty {
                overwrite_existing_files: true,
                overwrite_existing_subdirectories: false,
            },
            ..Default::default()
        },
        |_| {},
    );

    assert!(
        copy_result.is_err(),
        "copy_directory_with_progress should have errored due to existing target file"
    );

    let copy_err = copy_result.unwrap_err();
    match &copy_err {
        DirectoryError::TargetItemAlreadyExists { path } => {
            assert_eq!(
                path,
                &replicated_foo_dir_path,
                "copy_directory_with_progress returned TargetItemAlreadyExists with incorrect inner path"
            );
        }
        _ => {
            panic!("copy_directory_with_progress should have errored with TargetItemAlreadyExists")
        }
    }


    harness.destroy()?;
    empty_harness.destroy()?;
    Ok(())
}



#[test]
pub fn disallow_copy_directory_into_itself() -> TestResult<()> {
    let harness = DeepTreeHarness::new()?;

    let copy_result = fs_more::directory::copy_directory(
        harness.root.path(),
        harness.root.path(),
        DirectoryCopyOptions {
            target_directory_rule: TargetDirectoryRule::AllowNonEmpty {
                overwrite_existing_subdirectories: true,
                overwrite_existing_files: true,
            },
            ..Default::default()
        },
    );

    assert_matches!(
        copy_result,
        Err(DirectoryError::InvalidTargetDirectoryPath),
        "copy_directory should have errored when trying to copy a directory into itself"
    );

    harness.destroy()?;
    Ok(())
}

#[test]
pub fn disallow_copy_directory_into_subdirectory_of_itself() -> TestResult<()> {
    let harness = DeepTreeHarness::new()?;

    let copy_result = fs_more::directory::copy_directory(
        harness.root.path(),
        harness.dir_world.path(),
        DirectoryCopyOptions {
            target_directory_rule: TargetDirectoryRule::AllowNonEmpty {
                overwrite_existing_subdirectories: true,
                overwrite_existing_files: true,
            },
            ..Default::default()
        },
    );

    assert_matches!(
        copy_result,
        Err(DirectoryError::InvalidTargetDirectoryPath),
        "copy_directory should have errored when trying to \
        copy a directory into a subdirectory of itself"
    );

    harness.destroy()?;
    Ok(())
}

#[test]
pub fn disallow_copy_directory_with_progress_into_itself() -> TestResult<()> {
    let harness = DeepTreeHarness::new()?;

    let copy_result = fs_more::directory::copy_directory_with_progress(
        harness.root.path(),
        harness.root.path(),
        DirectoryCopyWithProgressOptions {
            target_directory_rule: TargetDirectoryRule::AllowNonEmpty {
                overwrite_existing_subdirectories: true,
                overwrite_existing_files: true,
            },
            ..Default::default()
        },
        |_| {},
    );

    assert_matches!(
        copy_result,
        Err(DirectoryError::InvalidTargetDirectoryPath),
        "copy_directory_with_progress should have errored when trying to \
        copy a directory into itself"
    );

    harness.destroy()?;
    Ok(())
}

#[test]
pub fn disallow_copy_directory_with_progress_into_subdirectory_of_itself() -> TestResult<()> {
    let harness = DeepTreeHarness::new()?;

    let copy_result = fs_more::directory::copy_directory_with_progress(
        harness.root.path(),
        harness.dir_world.path(),
        DirectoryCopyWithProgressOptions {
            target_directory_rule: TargetDirectoryRule::AllowNonEmpty {
                overwrite_existing_subdirectories: true,
                overwrite_existing_files: true,
            },
            ..Default::default()
        },
        |_| {},
    );

    assert_matches!(
        copy_result,
        Err(DirectoryError::InvalidTargetDirectoryPath),
        "copy_directory_with_progress should have errored when trying to \
        copy a directory into a subdirectory of itself"
    );

    harness.destroy()?;
    Ok(())
}

#[test]
pub fn error_on_copy_directory_on_existing_target_directory_without_option() -> TestResult<()> {
    let harness = DeepTreeHarness::new()?;
    let empty_harness = EmptyTreeHarness::new()?;

    empty_harness.root.assert_is_empty();

    let copy_result = fs_more::directory::copy_directory(
        harness.root.path(),
        empty_harness.root.path(),
        DirectoryCopyOptions {
            target_directory_rule: TargetDirectoryRule::DisallowExisting,
            ..Default::default()
        },
    );

    let copy_err = copy_result.unwrap_err();
    match &copy_err {
        DirectoryError::TargetItemAlreadyExists { path } => {
            assert_eq!(
                path,
                empty_harness.root.path(),
                "copy_directory did not return the correct path \
                inside the TargetItemAlreadyExists error"
            );
        }
        _ => panic!("Unexpected Err value: {}", copy_err),
    }

    empty_harness.root.assert_is_empty();

    harness.destroy()?;
    empty_harness.destroy()?;
    Ok(())
}

#[test]
pub fn error_on_copy_directory_on_existing_file_without_option() -> TestResult<()> {
    let harness = DeepTreeHarness::new()?;
    let empty_harness = EmptyTreeHarness::new()?;

    // Still the harness setup.
    let file_a_filename = harness.file_a.path().file_name().unwrap();
    let test_file_path = empty_harness.root.child_path(file_a_filename);
    fs_more::file::copy_file(
        harness.file_a.path(),
        &test_file_path,
        FileCopyOptions {
            overwrite_existing: false,
            skip_existing: false,
        },
    )
    .unwrap();

    let test_file = AssertableFilePath::from_path_with_captured_content(test_file_path)?;

    test_file.assert_exists();
    test_file.assert_content_unchanged();
    // End of setup, we have now pre-copied a single file to test our overwriting options.


    empty_harness.root.assert_is_not_empty();

    let copy_result = fs_more::directory::copy_directory(
        harness.root.path(),
        empty_harness.root.path(),
        DirectoryCopyOptions {
            target_directory_rule: TargetDirectoryRule::AllowNonEmpty {
                overwrite_existing_files: false,
                overwrite_existing_subdirectories: false,
            },
            ..Default::default()
        },
    );

    let copy_err = copy_result.unwrap_err();
    match &copy_err {
        DirectoryError::TargetItemAlreadyExists { path } => {
            assert_eq!(
                path,
                test_file.path(),
                "copy_directory did not return the correct path \
                inside the TargetItemAlreadyExists error"
            );
        }
        _ => panic!("Unexpected Err value: {}", copy_err),
    }

    empty_harness.root.assert_is_not_empty();

    harness.destroy()?;
    empty_harness.destroy()?;
    Ok(())
}

#[test]
pub fn error_on_copy_directory_on_existing_subdirectory_without_option() -> TestResult<()> {
    let harness = DeepTreeHarness::new()?;
    let empty_harness = EmptyTreeHarness::new()?;

    // Still the harness setup.
    let replicated_foo_dir_name = harness.dir_foo.path().file_name().unwrap();
    let replicated_foo_dir_path = empty_harness.root.child_path(replicated_foo_dir_name);
    std::fs::create_dir_all(&replicated_foo_dir_path)?;

    let file_b_filename = harness.file_b.path().file_name().unwrap();
    let replicated_file_b_path = empty_harness.root.child_path(file_b_filename);
    fs_more::file::copy_file(
        harness.file_b.path(),
        &replicated_file_b_path,
        FileCopyOptions {
            overwrite_existing: false,
            skip_existing: false,
        },
    )
    .unwrap();

    let replicated_file_b =
        AssertableFilePath::from_path_with_captured_content(replicated_file_b_path)?;

    replicated_file_b.assert_exists();
    replicated_file_b.assert_content_unchanged();
    // End of setup, we have now pre-copied a single directory containing
    // a single file to test our overwriting options.


    empty_harness.root.assert_is_not_empty();

    let copy_result = fs_more::directory::copy_directory(
        harness.root.path(),
        empty_harness.root.path(),
        DirectoryCopyOptions {
            target_directory_rule: TargetDirectoryRule::AllowNonEmpty {
                overwrite_existing_files: true,
                overwrite_existing_subdirectories: false,
            },
            ..Default::default()
        },
    );

    let copy_err = copy_result.unwrap_err();
    match &copy_err {
        DirectoryError::TargetItemAlreadyExists { path } => {
            assert_eq!(
                path, &replicated_foo_dir_path,
                "copy_directory did not return the correct path \
                inside the TargetItemAlreadyExists error"
            );
        }
        _ => panic!("Unexpected Err value: {}", copy_err),
    }

    empty_harness.root.assert_is_not_empty();

    harness.destroy()?;
    empty_harness.destroy()?;
    Ok(())
}


#[test]
pub fn copy_directory_symbolic_link_to_file_behaviour() -> TestResult<()> {
    let harness = DeepTreeHarness::new()?;
    let empty_harness = EmptyTreeHarness::new()?;

    let symlinked_file =
        AssertableFilePath::from_path(harness.root.child_path("file_a-symlinked.bin"));
    symlinked_file.assert_not_exists();
    symlinked_file.symlink_to_file(harness.file_a.path())?;
    symlinked_file.assert_is_symlink_to_file();

    fs_more::directory::copy_directory(
        harness.root.path(),
        empty_harness.root.path(),
        DirectoryCopyOptions::default(),
    )
    .unwrap();

    let previously_symlinked_file_in_target =
        AssertableFilePath::from_path(empty_harness.root.child_path("file_a-symlinked.bin"));
    previously_symlinked_file_in_target.assert_exists();
    previously_symlinked_file_in_target.assert_is_file();

    empty_harness.destroy()?;
    harness.destroy()?;
    Ok(())
}

#[test]
pub fn copy_directory_with_progress_symbolic_link_to_file_behaviour() -> TestResult<()> {
    let harness = DeepTreeHarness::new()?;
    let empty_harness = EmptyTreeHarness::new()?;

    let symlinked_file =
        AssertableFilePath::from_path(harness.root.child_path("file_a-symlinked.bin"));
    symlinked_file.assert_not_exists();
    symlinked_file.symlink_to_file(harness.file_a.path())?;
    symlinked_file.assert_is_symlink_to_file();

    fs_more::directory::copy_directory_with_progress(
        harness.root.path(),
        empty_harness.root.path(),
        DirectoryCopyWithProgressOptions::default(),
        |_| {},
    )
    .unwrap();

    let previously_symlinked_file_in_target =
        AssertableFilePath::from_path(empty_harness.root.child_path("file_a-symlinked.bin"));
    previously_symlinked_file_in_target.assert_exists();
    previously_symlinked_file_in_target.assert_is_file();

    empty_harness.destroy()?;
    harness.destroy()?;
    Ok(())
}


#[test]
pub fn copy_directory_symbolic_link_to_directory_behaviour() -> TestResult<()> {
    let harness = DeepTreeHarness::new()?;
    let empty_harness = EmptyTreeHarness::new()?;

    let symlinked_dir =
        AssertableDirectoryPath::from_path(harness.root.child_path("symlinked-directory"));
    symlinked_dir.assert_not_exists();
    symlinked_dir.symlink_to_directory(harness.dir_foo.path())?;
    symlinked_dir.assert_is_symlink_to_directory();

    fs_more::directory::copy_directory(
        harness.root.path(),
        empty_harness.root.path(),
        DirectoryCopyOptions::default(),
    )
    .unwrap();

    let previously_symlinked_dir_in_target =
        AssertableDirectoryPath::from_path(empty_harness.root.child_path("symlinked-directory"));
    previously_symlinked_dir_in_target.assert_exists();
    previously_symlinked_dir_in_target
        .assert_directory_contents_match_directory(harness.dir_foo.path());

    empty_harness.destroy()?;
    harness.destroy()?;
    Ok(())
}

#[test]
pub fn copy_directory_with_progress_symbolic_link_to_directory_behaviour() -> TestResult<()> {
    let harness = DeepTreeHarness::new()?;
    let empty_harness = EmptyTreeHarness::new()?;

    let symlinked_dir =
        AssertableDirectoryPath::from_path(harness.root.child_path("symlinked-directory"));
    symlinked_dir.assert_not_exists();
    symlinked_dir.symlink_to_directory(harness.dir_foo.path())?;
    symlinked_dir.assert_is_symlink_to_directory();

    fs_more::directory::copy_directory_with_progress(
        harness.root.path(),
        empty_harness.root.path(),
        DirectoryCopyWithProgressOptions::default(),
        |_| {},
    )
    .unwrap();

    let previously_symlinked_dir_in_target =
        AssertableDirectoryPath::from_path(empty_harness.root.child_path("symlinked-directory"));
    previously_symlinked_dir_in_target.assert_exists();
    previously_symlinked_dir_in_target
        .assert_directory_contents_match_directory(harness.dir_foo.path());

    empty_harness.destroy()?;
    harness.destroy()?;
    Ok(())
}

#[test]
pub fn copy_directory_symbolic_link_to_directory_respect_depth_limit() -> TestResult<()> {
    let harness = DeepTreeHarness::new()?;
    let empty_harness = EmptyTreeHarness::new()?;

    let symlinked_dir =
        AssertableDirectoryPath::from_path(harness.root.child_path("symlinked-directory"));
    symlinked_dir.assert_not_exists();
    symlinked_dir.symlink_to_directory(harness.dir_foo.path())?;
    symlinked_dir.assert_is_symlink_to_directory();

    fs_more::directory::copy_directory(
        harness.root.path(),
        empty_harness.root.path(),
        DirectoryCopyOptions {
            maximum_copy_depth: Some(1),
            ..Default::default()
        },
    )
    .unwrap();

    let previously_symlinked_dir_in_target =
        AssertableDirectoryPath::from_path(empty_harness.root.child_path("symlinked-directory"));
    previously_symlinked_dir_in_target.assert_exists();

    let previously_symlinked_file_b = AssertableFilePath::from_path(
        empty_harness
            .root
            .child_path("symlinked-directory")
            .join(harness.file_b.path().file_name().unwrap()),
    );
    previously_symlinked_file_b.assert_is_file();
    previously_symlinked_file_b.assert_content_matches_file(harness.file_b.path());

    let previously_symlinked_file_c = AssertableFilePath::from_path(
        empty_harness
            .root
            .child_path("symlinked-directory")
            .join(harness.dir_bar.path().file_name().unwrap())
            .join(harness.file_c.path().file_name().unwrap()),
    );
    previously_symlinked_file_c.assert_not_exists();

    empty_harness.destroy()?;
    harness.destroy()?;
    Ok(())
}

#[test]
pub fn copy_directory_with_progress_symbolic_link_to_directory_respect_depth_limit(
) -> TestResult<()> {
    let harness = DeepTreeHarness::new()?;
    let empty_harness = EmptyTreeHarness::new()?;

    let symlinked_dir =
        AssertableDirectoryPath::from_path(harness.root.child_path("symlinked-directory"));
    symlinked_dir.assert_not_exists();
    symlinked_dir.symlink_to_directory(harness.dir_foo.path())?;
    symlinked_dir.assert_is_symlink_to_directory();

    fs_more::directory::copy_directory_with_progress(
        harness.root.path(),
        empty_harness.root.path(),
        DirectoryCopyWithProgressOptions {
            maximum_copy_depth: Some(1),
            ..Default::default()
        },
        |_| {},
    )
    .unwrap();

    let previously_symlinked_dir_in_target =
        AssertableDirectoryPath::from_path(empty_harness.root.child_path("symlinked-directory"));
    previously_symlinked_dir_in_target.assert_exists();

    let previously_symlinked_file_b = AssertableFilePath::from_path(
        empty_harness
            .root
            .child_path("symlinked-directory")
            .join(harness.file_b.path().file_name().unwrap()),
    );
    previously_symlinked_file_b.assert_is_file();
    previously_symlinked_file_b.assert_content_matches_file(harness.file_b.path());

    let previously_symlinked_file_c = AssertableFilePath::from_path(
        empty_harness
            .root
            .child_path("symlinked-directory")
            .join(harness.dir_bar.path().file_name().unwrap())
            .join(harness.file_c.path().file_name().unwrap()),
    );
    previously_symlinked_file_c.assert_not_exists();

    empty_harness.destroy()?;
    harness.destroy()?;
    Ok(())
}


#[test]
pub fn copy_directory_preemptively_check_for_directory_collisions() -> TestResult<()> {
    let harness = DeepTreeHarness::new()?;
    let empty_harness = EmptyTreeHarness::new()?;
    empty_harness.root.assert_is_empty();

    // Target directory preparation.
    let existing_target_file = AssertableFilePath::from_path(
        empty_harness.root.path().join(
            harness
                .file_d
                .path()
                .strip_prefix(harness.root.path())
                .unwrap(),
        ),
    );

    let non_existing_target_file = AssertableFilePath::from_path(
        empty_harness.root.path().join(
            harness
                .file_a
                .path()
                .strip_prefix(harness.root.path())
                .unwrap(),
        ),
    );

    std::fs::create_dir_all(existing_target_file.path().parent().unwrap()).unwrap();
    std::fs::copy(harness.file_d.path(), existing_target_file.path()).unwrap();

    existing_target_file.assert_content_matches_file(harness.file_d.path());
    non_existing_target_file.assert_not_exists();

    empty_harness.root.assert_is_not_empty();
    // END of preparation

    let copy_result = fs_more::directory::copy_directory(
        harness.root.path(),
        empty_harness.root.path(),
        DirectoryCopyOptions {
            target_directory_rule: TargetDirectoryRule::AllowNonEmpty {
                overwrite_existing_subdirectories: false,
                overwrite_existing_files: false,
            },
            ..Default::default()
        },
    );

    let copy_err = copy_result.unwrap_err();

    match copy_err {
        DirectoryError::TargetItemAlreadyExists { path } => {
            assert_eq!(
                path.as_path(),
                empty_harness.root.path()
                    .join(
                        harness.dir_foo.path()
                            .strip_prefix(harness.root.path())
                            .unwrap()
                    ),
                "copy_directory did not return the proper directory collision in the target directory"
            );
        }
        _ => panic!("Unexpected Err: {}", copy_err),
    }

    empty_harness.root.assert_is_not_empty();

    non_existing_target_file.assert_not_exists();
    existing_target_file.assert_content_matches_file(harness.file_d.path());


    harness.destroy()?;
    empty_harness.destroy()?;
    Ok(())
}

#[test]
pub fn copy_directory_preemptively_check_for_file_collisions() -> TestResult<()> {
    let harness = DeepTreeHarness::new()?;
    let empty_harness = EmptyTreeHarness::new()?;
    empty_harness.root.assert_is_empty();

    // Target directory preparation.
    let existing_target_file = AssertableFilePath::from_path(
        empty_harness.root.path().join(
            harness
                .file_d
                .path()
                .strip_prefix(harness.root.path())
                .unwrap(),
        ),
    );

    let non_existing_target_file = AssertableFilePath::from_path(
        empty_harness.root.path().join(
            harness
                .file_a
                .path()
                .strip_prefix(harness.root.path())
                .unwrap(),
        ),
    );

    std::fs::create_dir_all(existing_target_file.path().parent().unwrap()).unwrap();
    std::fs::copy(harness.file_d.path(), existing_target_file.path()).unwrap();

    existing_target_file.assert_content_matches_file(harness.file_d.path());
    non_existing_target_file.assert_not_exists();

    empty_harness.root.assert_is_not_empty();
    // END of preparation

    let copy_result = fs_more::directory::copy_directory(
        harness.root.path(),
        empty_harness.root.path(),
        DirectoryCopyOptions {
            target_directory_rule: TargetDirectoryRule::AllowNonEmpty {
                overwrite_existing_subdirectories: true,
                overwrite_existing_files: false,
            },
            ..Default::default()
        },
    );

    let copy_err = copy_result.unwrap_err();

    match copy_err {
        DirectoryError::TargetItemAlreadyExists { path } => {
            assert_eq!(
                path.as_path(),
                existing_target_file.path(),
                "copy_directory did not return the proper file collision in the target directory"
            );
        }
        _ => panic!("Unexpected Err: {}", copy_err),
    }

    empty_harness.root.assert_is_not_empty();

    non_existing_target_file.assert_not_exists();
    existing_target_file.assert_content_matches_file(harness.file_d.path());


    harness.destroy()?;
    empty_harness.destroy()?;
    Ok(())
}


#[test]
pub fn copy_directory_with_progress_preemptively_check_for_directory_collisions() -> TestResult<()>
{
    let harness = DeepTreeHarness::new()?;
    let empty_harness = EmptyTreeHarness::new()?;
    empty_harness.root.assert_is_empty();

    // Target directory preparation.
    let existing_target_file = AssertableFilePath::from_path(
        empty_harness.root.path().join(
            harness
                .file_d
                .path()
                .strip_prefix(harness.root.path())
                .unwrap(),
        ),
    );

    let non_existing_target_file = AssertableFilePath::from_path(
        empty_harness.root.path().join(
            harness
                .file_a
                .path()
                .strip_prefix(harness.root.path())
                .unwrap(),
        ),
    );

    std::fs::create_dir_all(existing_target_file.path().parent().unwrap()).unwrap();
    std::fs::copy(harness.file_d.path(), existing_target_file.path()).unwrap();

    existing_target_file.assert_content_matches_file(harness.file_d.path());
    non_existing_target_file.assert_not_exists();

    empty_harness.root.assert_is_not_empty();
    // END of preparation

    let mut last_progress: Option<DirectoryCopyProgress> = None;

    let copy_result = fs_more::directory::copy_directory_with_progress(
        harness.root.path(),
        empty_harness.root.path(),
        DirectoryCopyWithProgressOptions {
            target_directory_rule: TargetDirectoryRule::AllowNonEmpty {
                overwrite_existing_subdirectories: false,
                overwrite_existing_files: false,
            },
            ..Default::default()
        },
        |progress| {
            last_progress = Some(progress.clone());
        },
    );

    let copy_err = copy_result.unwrap_err();

    assert!(
        last_progress.is_none(),
        "copy_directory_with_progress did not check for directory collisions before starting copy"
    );

    match copy_err {
        DirectoryError::TargetItemAlreadyExists { path } => {
            assert_eq!(
                path.as_path(),
                empty_harness.root.path()
                    .join(
                        harness.dir_foo.path()
                            .strip_prefix(harness.root.path())
                            .unwrap()
                    ),
                "copy_directory did not return the proper directory collision in the target directory"
            );
        }
        _ => panic!("Unexpected Err: {}", copy_err),
    }

    empty_harness.root.assert_is_not_empty();

    non_existing_target_file.assert_not_exists();
    existing_target_file.assert_content_matches_file(harness.file_d.path());


    harness.destroy()?;
    empty_harness.destroy()?;
    Ok(())
}

#[test]
pub fn copy_directory_with_progress_preemptively_check_for_file_collisions() -> TestResult<()> {
    let harness = DeepTreeHarness::new()?;
    let empty_harness = EmptyTreeHarness::new()?;
    empty_harness.root.assert_is_empty();

    // Target directory preparation.
    let existing_target_file = AssertableFilePath::from_path(
        empty_harness.root.path().join(
            harness
                .file_d
                .path()
                .strip_prefix(harness.root.path())
                .unwrap(),
        ),
    );

    let non_existing_target_file = AssertableFilePath::from_path(
        empty_harness.root.path().join(
            harness
                .file_a
                .path()
                .strip_prefix(harness.root.path())
                .unwrap(),
        ),
    );

    std::fs::create_dir_all(existing_target_file.path().parent().unwrap()).unwrap();
    std::fs::copy(harness.file_d.path(), existing_target_file.path()).unwrap();

    existing_target_file.assert_content_matches_file(harness.file_d.path());
    non_existing_target_file.assert_not_exists();

    empty_harness.root.assert_is_not_empty();
    // END of preparation

    let mut last_progress: Option<DirectoryCopyProgress> = None;

    let copy_result = fs_more::directory::copy_directory_with_progress(
        harness.root.path(),
        empty_harness.root.path(),
        DirectoryCopyWithProgressOptions {
            target_directory_rule: TargetDirectoryRule::AllowNonEmpty {
                overwrite_existing_subdirectories: true,
                overwrite_existing_files: false,
            },
            ..Default::default()
        },
        |progress| {
            last_progress = Some(progress.clone());
        },
    );

    let copy_err = copy_result.unwrap_err();

    assert!(
        last_progress.is_none(),
        "copy_directory_with_progress did not check for directory collisions before starting copy"
    );

    match copy_err {
        DirectoryError::TargetItemAlreadyExists { path } => {
            assert_eq!(
                path.as_path(),
                existing_target_file.path(),
                "copy_directory did not return the proper file collision in the target directory"
            );
        }
        _ => panic!("Unexpected Err: {}", copy_err),
    }

    empty_harness.root.assert_is_not_empty();

    non_existing_target_file.assert_not_exists();
    existing_target_file.assert_content_matches_file(harness.file_d.path());


    harness.destroy()?;
    empty_harness.destroy()?;
    Ok(())
}


#[test]
pub fn disallow_copy_directory_when_source_is_symlink_to_target() -> TestResult<()> {
    // Tests behaviour when copying "symlink to directory A" to "A".
    // This should fail.

    let harness_for_comparison = DeepTreeHarness::new()?;
    let harness = DeepTreeHarness::new()?;
    let intermediate_harness = EmptyTreeHarness::new()?;

    // Directory symlink preparation
    let symlink_to_directory = AssertableDirectoryPath::from_path(
        intermediate_harness.root.child_path("symlinked-directory"),
    );

    symlink_to_directory.assert_not_exists();
    symlink_to_directory
        .symlink_to_directory(harness.root.path())
        .unwrap();
    // END of preparation

    let copy_result = fs_more::directory::copy_directory(
        symlink_to_directory.path(),
        harness.root.path(),
        DirectoryCopyOptions {
            target_directory_rule: TargetDirectoryRule::AllowNonEmpty {
                overwrite_existing_subdirectories: true,
                overwrite_existing_files: true,
            },
            ..Default::default()
        },
    );

    let copy_err = copy_result.unwrap_err();

    match &copy_err {
        DirectoryError::InvalidTargetDirectoryPath => {
            // This is the expected error value.
        }
        _ => panic!("Unexpected Err: {}", copy_err),
    }

    harness
        .root
        .assert_directory_contents_match_directory(harness_for_comparison.root.path());


    harness_for_comparison.destroy()?;
    harness.destroy()?;
    intermediate_harness.destroy()?;

    Ok(())
}


#[test]
pub fn disallow_copy_directory_with_progress_when_source_is_symlink_to_target() -> TestResult<()> {
    // Tests behaviour when copying "symlink to directory A" to "A".
    // This should fail.

    let harness_for_comparison = DeepTreeHarness::new()?;
    let harness = DeepTreeHarness::new()?;
    let intermediate_harness = EmptyTreeHarness::new()?;

    // Directory symlink preparation
    let symlink_to_directory = AssertableDirectoryPath::from_path(
        intermediate_harness.root.child_path("symlinked-directory"),
    );

    symlink_to_directory.assert_not_exists();
    symlink_to_directory
        .symlink_to_directory(harness.root.path())
        .unwrap();
    // END of preparation

    let mut last_progress: Option<DirectoryCopyProgress> = None;

    let copy_result = fs_more::directory::copy_directory_with_progress(
        symlink_to_directory.path(),
        harness.root.path(),
        DirectoryCopyWithProgressOptions {
            target_directory_rule: TargetDirectoryRule::AllowNonEmpty {
                overwrite_existing_subdirectories: true,
                overwrite_existing_files: true,
            },
            ..Default::default()
        },
        |progress| {
            last_progress = Some(progress.clone());
        },
    );

    assert!(last_progress.is_none());

    let copy_err = copy_result.unwrap_err();

    match &copy_err {
        DirectoryError::InvalidTargetDirectoryPath => {
            // This is the expected error value.
        }
        _ => panic!("Unexpected Err: {}", copy_err),
    }

    harness
        .root
        .assert_directory_contents_match_directory(harness_for_comparison.root.path());


    harness_for_comparison.destroy()?;
    harness.destroy()?;
    intermediate_harness.destroy()?;

    Ok(())
}
