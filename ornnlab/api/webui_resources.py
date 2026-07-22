from fastapi import APIRouter

from ornnlab.api.webui_job_deletion import router as job_deletion_router
from ornnlab.api.webui_model_pricing import router as model_pricing_router

router = APIRouter()
router.include_router(job_deletion_router)
router.include_router(model_pricing_router)
