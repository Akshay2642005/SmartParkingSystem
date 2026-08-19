# SPDX-FileCopyrightText: 2022-2025 Espressif Systems (Shanghai) CO LTD
# SPDX-License-Identifier: CC0-1.0
from typing import Callable

import pytest
from pytest_embedded_idf.dut import IdfDut
from pytest_embedded_idf.utils import idf_parametrize


@pytest.mark.generic
@idf_parametrize('target', ['supported_targets', 'preview_targets'], indirect=['target'])
def test_smart_parking(dut: IdfDut, log_minimum_free_heap_size: Callable[..., None]) -> None:
    dut.expect('Smart Parking System')
    dut.expect('Initialized 3 parking slots')
    log_minimum_free_heap_size()


@pytest.mark.host_test
@idf_parametrize('target', ['linux'], indirect=['target'])
def test_smart_parking_linux(dut: IdfDut) -> None:
    dut.expect('Smart Parking System')


@pytest.mark.host_test
@pytest.mark.macos
@idf_parametrize('target', ['linux'], indirect=['target'])
def test_smart_parking_macos(dut: IdfDut) -> None:
    dut.expect('Smart Parking System')