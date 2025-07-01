// SPDX-License-Identifier: Apache-2.0
// SPDX-FileCopyrightText: 2025 Ignacy Kajdan <ignacy.kajdan@grinn-global.com>

use debian_genio_flash::interrupt::InterruptState;

#[tokio::test]
async fn sets_interrupt_and_abort_flags() {
    let state = InterruptState::new();
    assert!(!state.interrupted);
    assert!(!state.confirmed_abort);
}
