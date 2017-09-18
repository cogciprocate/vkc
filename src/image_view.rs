
use std::sync::Arc;
use std::ptr;
use vk;
use ::{VkcResult, Swapchain};


#[derive(Debug)]
pub struct Inner {
    handle: vk::ImageView,
    swapchain: Swapchain,
}

#[derive(Debug, Clone)]
pub struct ImageView {
    inner: Arc<Inner>,
}

impl ImageView {
    pub fn new(swapchain: Swapchain, image: vk::Image) -> VkcResult<ImageView> {
        let create_info = vk::ImageViewCreateInfo {
            sType: vk::STRUCTURE_TYPE_IMAGE_VIEW_CREATE_INFO,
            pNext: ptr::null(),
            flags: 0,
            image: image,
            viewType: vk::IMAGE_VIEW_TYPE_2D,
            format: swapchain.image_format(),
            components: vk::ComponentMapping {
                r: vk::COMPONENT_SWIZZLE_IDENTITY,
                g: vk::COMPONENT_SWIZZLE_IDENTITY,
                b: vk::COMPONENT_SWIZZLE_IDENTITY,
                a: vk::COMPONENT_SWIZZLE_IDENTITY,
            },
            subresourceRange: vk::ImageSubresourceRange {
                aspectMask: vk::IMAGE_ASPECT_COLOR_BIT,
                baseMipLevel: 0,
                levelCount: 1,
                baseArrayLayer: 0,
                layerCount: 1,
            },
        };

        let mut handle = 0;

        unsafe {
            ::check(swapchain.device().vk().CreateImageView(swapchain.device().handle(),
                &create_info, ptr::null(), &mut handle));
        }

        Ok(ImageView {
            inner: Arc::new(Inner {
                handle: handle,
                swapchain: swapchain,
            })
        })
    }

    pub fn handle(&self) -> vk::ImageView {
        self.inner.handle
    }

    pub fn swapchain(&self) -> &Swapchain {
        &self.inner.swapchain
    }
}

impl Drop for Inner {
    fn drop(&mut self) {
        unsafe {
            self.swapchain.device().vk().DestroyImageView(self.swapchain.device().handle(),
                self.handle, ptr::null());
        }
    }
}


pub fn create_image_views(swapchain: &Swapchain) -> VkcResult<Vec<ImageView>> {
    // let mut image_views: Vec<ImageView> = Vec::with_capacity(swapchain.images().len());
    // for &image in swapchain.images() {
    //     image_views.push(ImageView::new(swapchain.clone(), image)?);
    // }
    // Ok(image_views)
    swapchain.images().iter().map(|&image| {
        ImageView::new(swapchain.clone(), image)
    }).collect::<Result<Vec<_>, _>>()
}
