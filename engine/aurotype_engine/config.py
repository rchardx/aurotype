"""Configuration for Aurotype Engine via environment variables."""

from typing import Optional
from pydantic_settings import BaseSettings, SettingsConfigDict


class Settings(BaseSettings):
    """Load configuration from environment variables with AUROTYPE_ prefix."""

    stt_provider: str = "aliyun_dashscope"
    llm_provider: str = "deepseek"
    stt_model: Optional[str] = None
    openai_api_key: Optional[str] = None
    deepseek_api_key: Optional[str] = None
    aliyun_dashscope_api_key: Optional[str] = None
    llm_base_url: Optional[str] = None
    llm_model: Optional[str] = None
    system_prompt: Optional[str] = None
    language: str = "auto"

    model_config = SettingsConfigDict(env_prefix="AUROTYPE_")


def get_settings() -> Settings:
    """Get the application settings singleton."""
    return Settings()
