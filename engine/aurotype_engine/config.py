"""Configuration for Aurotype Engine via environment variables."""

from typing import Optional
from pydantic_settings import BaseSettings, SettingsConfigDict


class Settings(BaseSettings):
    """Load configuration from environment variables with AUROTYPE_ prefix."""

    stt_provider: str = "deepgram"
    llm_provider: str = "openai"
    deepgram_api_key: Optional[str] = None
    openai_api_key: Optional[str] = None
    siliconflow_api_key: Optional[str] = None
    language: str = "auto"

    model_config = SettingsConfigDict(env_prefix="AUROTYPE_")


def get_settings() -> Settings:
    """Get the application settings singleton."""
    return Settings()
