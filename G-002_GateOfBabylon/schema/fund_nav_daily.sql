CREATE TABLE IF NOT EXISTS public.fund_nav_daily (
  fund_id VARCHAR(16) NOT NULL,
  nav_date DATE NOT NULL,
  net_asset_value NUMERIC(6, 3) NOT NULL,
  accumulated_asset_value NUMERIC(6, 3) NOT NULL,
  _create_timestamp TIMESTAMPTZ NOT NULL DEFAULT NOW(),
  _update_timestamp TIMESTAMPTZ NOT NULL DEFAULT NOW(),
  PRIMARY KEY (fund_id, nav_date)
);

CREATE OR REPLACE FUNCTION public.set_fund_nav_daily_updated_timestamp()
RETURNS TRIGGER
LANGUAGE plpgsql
AS $$
BEGIN
  NEW._update_timestamp := NOW();
  RETURN NEW;
END;
$$;

DROP TRIGGER IF EXISTS fund_nav_daily_set_updated ON public.fund_nav_daily;
CREATE TRIGGER fund_nav_daily_set_updated
BEFORE UPDATE ON public.fund_nav_daily
FOR EACH ROW
EXECUTE FUNCTION public.set_fund_nav_daily_updated_timestamp();
