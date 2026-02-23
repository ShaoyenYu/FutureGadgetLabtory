CREATE TABLE IF NOT EXISTS public.fund_info (
  fund_id VARCHAR(16) PRIMARY KEY,
  fund_name VARCHAR(64) NOT NULL,
  _create_timestamp TIMESTAMPTZ NOT NULL DEFAULT NOW(),
  _update_timestamp TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE OR REPLACE FUNCTION public.set_fund_info_updated_timestamp()
RETURNS TRIGGER
LANGUAGE plpgsql
AS $$
BEGIN
  NEW._update_timestamp := NOW();
  RETURN NEW;
END;
$$;

DROP TRIGGER IF EXISTS fund_info_set_updated ON public.fund_info;
CREATE TRIGGER fund_info_set_updated
BEFORE UPDATE ON public.fund_info
FOR EACH ROW
EXECUTE FUNCTION public.set_fund_info_updated_timestamp();
