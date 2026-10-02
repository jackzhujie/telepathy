use once_cell::sync::Lazy;
use reqwest::Client;
use std::sync::RwLock;
use std::time::{Duration, Instant};
use tokio::time::timeout;

static GEO_CACHE: Lazy<RwLock<Option<bool>>> = Lazy::new(|| RwLock::new(None));

pub struct GeoSensor;

impl GeoSensor {
    pub async fn is_likely_in_china() -> bool {
        if let Some(cached) = *GEO_CACHE.read().unwrap() {
            return cached;
        }

        let client = Client::builder()
            .timeout(Duration::from_secs(2))
            .user_agent("Telepathy/1.0")
            .build()
            .unwrap_or_default();

        // 优先探测 ModelScope，如果极快且网络连通，推测为国内环境
        let _start = Instant::now();
        let ms_latency = Self::probe(&client, "https://modelscope.cn/api/v1/models").await;

        let is_china = if let Some(ms_lat) = ms_latency {
            if ms_lat < 500 {
                // MS 响应很快，极大概率在国内，直接锁定，不再浪费时间去探测 HF
                println!(
                    "[ModelHub] Geo-sensing: MS responds fast ({:?}ms). Locking to ModelScope.",
                    ms_lat
                );
                true
            } else {
                // MS 慢或一般，再看看 HF 表现
                let hf_latency = Self::probe(&client, "https://huggingface.co/api/models").await;
                println!(
                    "[ModelHub] Geo-sensing: MS: {:?}ms, HF: {:?}ms",
                    ms_latency, hf_latency
                );
                match hf_latency {
                    None => true,                        // HF 连不通：国内
                    Some(hf_lat) => hf_lat > ms_lat * 2, // HF 明显慢于 MS：国内
                }
            }
        } else {
            // MS 连不通：海外或网络异常，尝试 HF
            let hf_latency = Self::probe(&client, "https://huggingface.co/api/models").await;
            println!("[ModelHub] Geo-sensing: MS failed. HF: {:?}ms", hf_latency);
            hf_latency.is_none() // 两边都挂了默认为国内(保守)，或者 HF 通了则为海外
        };

        println!(
            "[ModelHub] Geo-sensing complete. User in China? -> {}",
            is_china
        );
        *GEO_CACHE.write().unwrap() = Some(is_china);
        is_china
    }

    async fn probe(client: &Client, url: &str) -> Option<u128> {
        let start = Instant::now();
        // Since we don't have auth on these probe edges, 401/403/200 are all valid network success
        let res = timeout(Duration::from_secs(2), client.head(url).send()).await;

        match res {
            Ok(Ok(response)) => {
                if !response.status().is_server_error() {
                    Some(start.elapsed().as_millis())
                } else {
                    None
                }
            }
            _ => None,
        }
    }
}
