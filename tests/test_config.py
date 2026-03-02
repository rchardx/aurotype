# pyright: reportMissingImports=false, reportUnknownVariableType=false, reportUnknownMemberType=false, reportUnknownArgumentType=false, reportAny=false

import os
import sys
from pathlib import Path
from unittest.mock import patch

sys.path.insert(0, str(Path(__file__).resolve().parents[1] / "engine"))

from aurotype_engine.config import Settings, get_settings


def test_settings_default_stt_provider() -> None:
    """Settings defaults to aliyun_dashscope STT provider."""
    settings = Settings()
    assert settings.stt_provider == "aliyun_dashscope"


def test_settings_default_llm_provider() -> None:
    """Settings defaults to deepseek LLM provider."""
    settings = Settings()
    assert settings.llm_provider == "deepseek"


def test_settings_default_language() -> None:
    """Settings defaults to auto language."""
    settings = Settings()
    assert settings.language == "auto"


def test_settings_default_optional_fields_are_none() -> None:
    """Optional fields default to None."""
    settings = Settings()
    assert settings.stt_model is None
    assert settings.openai_api_key is None
    assert settings.deepseek_api_key is None
    assert settings.aliyun_dashscope_api_key is None
    assert settings.llm_base_url is None
    assert settings.llm_model is None
    assert settings.system_prompt is None



def test_settings_env_override_llm_provider() -> None:
    """AUROTYPE_LLM_PROVIDER env var overrides default."""
    with patch.dict(os.environ, {"AUROTYPE_LLM_PROVIDER": "openai"}):
        settings = Settings()
    assert settings.llm_provider == "openai"


def test_settings_env_override_language() -> None:
    """AUROTYPE_LANGUAGE env var overrides default."""
    with patch.dict(os.environ, {"AUROTYPE_LANGUAGE": "zh"}):
        settings = Settings()
    assert settings.language == "zh"


def test_settings_env_override_api_keys() -> None:
    """AUROTYPE_ prefix env vars override API key fields."""
    with patch.dict(
        os.environ,
        {
            "AUROTYPE_ALIYUN_DASHSCOPE_API_KEY": "ds-123",
            "AUROTYPE_DEEPSEEK_API_KEY": "dk-456",
            "AUROTYPE_OPENAI_API_KEY": "sk-789",
        },
    ):
        settings = Settings()
    assert settings.aliyun_dashscope_api_key == "ds-123"
    assert settings.deepseek_api_key == "dk-456"
    assert settings.openai_api_key == "sk-789"


def test_get_settings_returns_settings_instance() -> None:
    """get_settings() returns a Settings instance."""
    settings = get_settings()
    assert isinstance(settings, Settings)


def test_settings_env_override_llm_base_url() -> None:
    """AUROTYPE_LLM_BASE_URL env var overrides default."""
    with patch.dict(
        os.environ, {"AUROTYPE_LLM_BASE_URL": "https://custom.example.com"}
    ):
        settings = Settings()
    assert settings.llm_base_url == "https://custom.example.com"
